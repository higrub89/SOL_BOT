//! # Price Feed — Hub Central de Precios
//!
//! Unifica múltiples fuentes de datos en un solo canal `tokio::mpsc`.
//! El monitor principal consume de aquí en lugar de hacer polling directo a APIs.
//!
//! ## Arquitectura (3 tiers)
//! ```text
//!   [Geyser gRPC]    ──push──▶ ┌──────────┐
//!   [WebSocket RPC]  ──push──▶ │ PriceFeed │ ──▶ mpsc::Receiver<PriceUpdate>
//!   [DexScreener]    ──pull──▶ └──────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

use crate::geyser::{GeyserClient, GeyserConfig};
use crate::scanner::PriceScanner;
use crate::telegram::TelegramNotifier;

/// Una actualización de precio normalizada, independiente de la fuente
#[derive(Debug, Clone)]
pub struct PriceUpdate {
    /// Mint address del token
    pub token_mint: String,
    /// Símbolo humano (ej: "ICEBEAR")
    pub symbol: String,
    /// Precio en USD
    pub price_usd: f64,
    /// Precio en SOL nativo (si disponible)
    pub price_native: f64,
    /// Liquidez del pool en USD
    pub liquidity_usd: f64,
    /// Volumen 24h en USD
    pub volume_24h: f64,
    /// Cambio de precio 24h en %
    pub price_change_24h: f64,
    /// De dónde vino este dato
    pub source: PriceSource,
    /// Timestamp de cuándo se recibió
    pub received_at: Instant,
}

/// Fuente del dato de precio
#[derive(Debug, Clone, PartialEq)]
pub enum PriceSource {
    Geyser,
    WebSocket,
    DexScreener,
}

/// Comandos para controlar el feed en caliente
#[derive(Debug, Clone)]
pub enum FeedCommand {
    /// Añadir un nuevo token al monitoreo
    Subscribe(MonitoredToken),
}

impl std::fmt::Display for PriceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceSource::Geyser => write!(f, "Geyser(gRPC)"),
            PriceSource::WebSocket => write!(f, "WebSocket(RPC)"),
            PriceSource::DexScreener => write!(f, "DexScreener(HTTP)"),
        }
    }
}

/// Información sobre un token que el PriceFeed debe monitorear
#[derive(Debug, Clone)]
pub struct MonitoredToken {
    pub mint: String,
    pub symbol: String,
    /// Pool account address para suscribirse en Geyser (si se conoce)
    pub pool_account: Option<String>,
    /// Coin vault (base token) del pool — para tracking de reserves
    pub coin_vault: Option<String>,
    /// PC vault (quote/SOL) del pool — para tracking de reserves
    pub pc_vault: Option<String>,
    /// Decimales del token (default: 6)
    pub token_decimals: u8,
}

/// Caché thread-safe del último precio conocido por token
pub type PriceCache = Arc<RwLock<HashMap<String, PriceUpdate>>>;

/// Configuración del PriceFeed
#[derive(Debug, Clone)]
pub struct PriceFeedConfig {
    /// Intervalo de polling de DexScreener (fallback)
    pub dexscreener_interval: Duration,
    /// Si Geyser está habilitado
    pub geyser_enabled: bool,
    /// Endpoint de Geyser (si aplica)
    pub geyser_endpoint: Option<String>,
    /// Token de autenticación (Helius API Key)
    pub geyser_token: Option<String>,
    /// Tiempo máximo sin recibir update de Geyser antes de escalar a DexScreener
    pub geyser_staleness_timeout: Duration,
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self {
            dexscreener_interval: Duration::from_secs(15),
            geyser_enabled: false,
            geyser_endpoint: None,
            geyser_token: None,
            geyser_staleness_timeout: Duration::from_secs(30),
        }
    }
}

impl PriceFeedConfig {
    /// Construye la configuración desde variables de entorno
    pub fn from_env() -> Self {
        // Filtrar strings vacíos: GEYSER_ENDPOINT= (vacío) debe ser None
        let geyser_endpoint = std::env::var("GEYSER_ENDPOINT")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let geyser_token = std::env::var("HELIUS_API_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty());

        let geyser_enabled = geyser_endpoint.is_some();

        let dex_interval_secs: u64 = std::env::var("DEXSCREENER_INTERVAL_SEC")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(if geyser_enabled { 30 } else { 5 });

        Self {
            dexscreener_interval: Duration::from_secs(dex_interval_secs),
            geyser_enabled,
            geyser_endpoint,
            geyser_token,
            geyser_staleness_timeout: Duration::from_secs(30),
        }
    }
}

/// El PriceFeed central. Lanza tareas en background y devuelve un Receiver unificado.
pub struct PriceFeed;

impl PriceFeed {
    /// Arranca el PriceFeed y devuelve:
    /// - Un `Receiver<PriceUpdate>` del cual el monitor principal consume
    /// - Un `PriceCache` con el último precio de cada token (para consultas rápidas)
    pub fn start(
        config: PriceFeedConfig,
        tokens: Vec<MonitoredToken>,
    ) -> (
        mpsc::Receiver<PriceUpdate>,
        PriceCache,
        mpsc::Sender<FeedCommand>,
    ) {
        let (tx, rx) = mpsc::channel::<PriceUpdate>(512);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<FeedCommand>(32);
        let cache: PriceCache = Arc::new(RwLock::new(HashMap::new()));
        let shared_tokens = Arc::new(RwLock::new(tokens.clone()));

        // ── Tarea de Gestión de Comandos (Dynamic Subscription) ──
        let cmd_tokens = Arc::clone(&shared_tokens);
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    FeedCommand::Subscribe(token) => {
                        println!(
                            "⚡ [Feed] Dynamic Subscription: ${} ({})",
                            token.symbol, token.mint
                        );
                        let mut t = cmd_tokens.write().await;
                        // Avoid duplicates
                        if !t.iter().any(|existing| existing.mint == token.mint) {
                            t.push(token);
                        }
                    }
                }
            }
        });

        // ── Tarea 1: DexScreener Poller ──
        let dex_tx = tx.clone();
        let dex_tokens = Arc::clone(&shared_tokens);
        let dex_interval = config.dexscreener_interval;
        let dex_cache = Arc::clone(&cache);

        tokio::spawn(async move {
            Self::dexscreener_loop(dex_tx, dex_tokens, dex_interval, dex_cache).await;
        });

        // ── Tarea 2: Geyser Streaming (si está habilitado) ──
        if config.geyser_enabled {
            if let (Some(endpoint), Some(token)) =
                (config.geyser_endpoint.clone(), config.geyser_token.clone())
            {
                let geyser_tx = tx.clone();
                let geyser_tokens = Arc::clone(&shared_tokens);
                let geyser_cache = Arc::clone(&cache);

                tokio::spawn(async move {
                    Self::geyser_stream_loop(
                        geyser_tx,
                        geyser_tokens,
                        endpoint,
                        token,
                        geyser_cache,
                    )
                    .await;
                });

                println!("   ⚡ Geyser gRPC: ACTIVADO (streaming en tiempo real)");
            } else {
                eprintln!("   ⚠️  Geyser habilitado pero falta GEYSER_ENDPOINT o HELIUS_API_KEY");
            }
        } else {
            // ── Tarea 2b: WebSocket nativo (alternativa GRATUITA a Geyser) ──
            // Se activa si hay vaults configuradas y tenemos API key para el WS
            let has_vaults = tokens
                .iter()
                .any(|t| t.coin_vault.is_some() && t.pc_vault.is_some());
            let helius_key = config.geyser_token.clone(); // Reusamos la Helius API key

            if has_vaults {
                if let Some(api_key) = helius_key {
                    let ws_url = format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key);
                    let ws_tx = tx.clone();
                    let ws_tokens = tokens.clone();
                    let ws_cache = Arc::clone(&cache);

                    tokio::spawn(async move {
                        crate::ws_feed::ws_price_loop(ws_tx, ws_tokens, ws_url, ws_cache).await;
                    });

                    println!("   🔌 WebSocket RPC: ACTIVADO (on-chain pricing GRATIS)");
                    println!("   💡 Upgrade: Configura GEYSER_ENDPOINT para latencia ultra-baja");
                } else {
                    println!("   📡 Geyser/WebSocket: DESACTIVADO (falta HELIUS_API_KEY)");
                    println!("   💡 Tip: Configura HELIUS_API_KEY en .env para activar WebSocket");
                }
            } else {
                println!("   📡 Geyser/WebSocket: DESACTIVADO (sin vault accounts)");
                println!("   💡 Tip: Usa 'cargo run --bin find_vaults' para configurar vaults");
            }
        }

        (rx, cache, cmd_tx)
    }

    /// Loop de polling a DexScreener (la fuente lenta pero fiable)
    async fn dexscreener_loop(
        tx: mpsc::Sender<PriceUpdate>,
        tokens_list: Arc<RwLock<Vec<MonitoredToken>>>,
        interval: Duration,
        cache: PriceCache,
    ) {
        let scanner = PriceScanner::new();

        loop {
            let tokens = {
                let r = tokens_list.read().await;
                r.clone()
            };

            for token in &tokens {
                match scanner.get_token_price(&token.mint).await {
                    Ok(price_data) => {
                        let update = PriceUpdate {
                            token_mint: token.mint.clone(),
                            symbol: token.symbol.clone(),
                            price_usd: price_data.price_usd,
                            price_native: price_data.price_native,
                            liquidity_usd: price_data.liquidity_usd,
                            volume_24h: price_data.volume_24h,
                            price_change_24h: price_data.price_change_24h,
                            source: PriceSource::DexScreener,
                            received_at: Instant::now(),
                        };

                        // Actualizar caché
                        {
                            let mut c = cache.write().await;
                            c.insert(token.mint.clone(), update.clone());
                        }

                        // Enviar al canal (non-blocking: si el buffer se llena, descartamos)
                        if tx.try_send(update).is_err() {
                            // Buffer lleno — el monitor no está consumiendo rápido.
                            // No es grave, el dato ya está en el caché.
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  [DexScreener] Error fetching {}: {}", token.symbol, e);
                    }
                }

                // Pequeña pausa entre tokens para no saturar la API
                tokio::time::sleep(Duration::from_millis(300)).await;
            }

            tokio::time::sleep(interval).await;
        }
    }

    /// Loop de streaming de Geyser gRPC (la fuente rápida) — V2 con cálculo de precio on-chain
    ///
    /// Estrategia: Se suscribe a las vault accounts (SPL Token Accounts) del pool.
    /// Cuando una vault cambia → parseamos su `amount` → actualizamos reserves → calculamos precio.
    ///
    /// ```text
    ///   Geyser update (vault account changed)
    ///     │
    ///     ▼
    ///   parse_spl_token_account_amount(data)  →  raw amount (u64)
    ///     │
    ///     ▼
    ///   VaultPair.update_reserve(vault_addr, amount)
    ///     │
    ///     ▼  (si ambas reserves están listas)
    ///   price_in_sol = pc_reserve / coin_reserve
    ///   price_in_usd = price_in_sol * sol_price_usd
    ///     │
    ///     ▼
    ///   PriceUpdate → mpsc channel → Monitor principal
    /// ```
    async fn geyser_stream_loop(
        tx: mpsc::Sender<PriceUpdate>,
        tokens_list: Arc<RwLock<Vec<MonitoredToken>>>,
        endpoint: String,
        api_token: String,
        cache: PriceCache,
    ) {
        use crate::amm_math::{
            build_vault_tracker, new_sol_price_tracker, parse_spl_token_account_amount,
            SolPriceUsd, VaultPair,
        };

        let mut reconnect_delay = Duration::from_secs(2);
        let max_reconnect_delay = Duration::from_secs(60);
        let mut reconnection_count: u32 = 0;
        let staleness_timeout = Duration::from_secs(45);
        let notifier = TelegramNotifier::new();

        loop {
            let tokens = {
                let r = tokens_list.read().await;
                r.clone()
            };

            // ── Construir VaultPair tracker ──
            let vault_pairs: Vec<VaultPair> = tokens
                .iter()
                .filter(|t| t.coin_vault.is_some() && t.pc_vault.is_some())
                .map(|t| VaultPair {
                    token_mint: t.mint.clone(),
                    symbol: t.symbol.clone(),
                    coin_vault: t.coin_vault.clone().unwrap(),
                    pc_vault: t.pc_vault.clone().unwrap(),
                    base_decimals: t.token_decimals,
                    quote_decimals: 9, // SOL siempre tiene 9 decimales
                    last_coin_reserve: None,
                    last_pc_reserve: None,
                })
                .collect();

            if vault_pairs.is_empty() {
                // Fallback: si no hay vaults configuradas, intentar con pool_account (modo legacy)
                let pool_accounts: Vec<String> = tokens
                    .iter()
                    .filter_map(|t| t.pool_account.clone())
                    .collect();

                if pool_accounts.is_empty() {
                    eprintln!("⚠️  [Geyser] No hay vault accounts ni pool accounts configurados.");
                    eprintln!("   💡 Añade 'coin_vault' + 'pc_vault' a tus targets");
                    return;
                }

                eprintln!("⚠️  [Geyser] Solo pool_account configurado (sin vaults).");
                eprintln!("   💡 Para cálculo de precio on-chain, configura coin_vault + pc_vault");
                // Podríamos implementar el modo legacy aquí, pero por ahora retornamos
                return;
            }

            let (vault_tracker, vault_to_mint) = build_vault_tracker(vault_pairs);

            // Todas las vault addresses que necesitamos monitorear
            let all_vault_addresses: Vec<String> = vault_to_mint.keys().cloned().collect();

            println!(
                "   🔑 Vault accounts a monitorear: {}",
                all_vault_addresses.len()
            );
            for addr in &all_vault_addresses {
                if let Some(mint) = vault_to_mint.get(addr) {
                    println!("      └─ {} → {}", &addr[..8], mint);
                }
            }

            // SOL price tracker (se actualiza desde el caché de DexScreener)
            let sol_price: SolPriceUsd = new_sol_price_tracker();

            loop {
                println!("🔌 [Geyser] Conectando a {}...", endpoint);

                let config = GeyserConfig {
                    endpoint: endpoint.clone(),
                    token: Some(api_token.clone()),
                };
                let client = GeyserClient::new(config);

                match client.connect().await {
                    Ok(mut grpc_client) => {
                        println!("✅ [Geyser] Conexión establecida");
                        reconnect_delay = Duration::from_secs(2);

                        // Notificar reconexión por Telegram (solo si es una reconexión, no la primera)
                        if reconnection_count > 0 {
                            let _ = notifier.send_connectivity_alert(
                                "Geyser gRPC",
                                true,
                                &format!("<b>Reconexión #{}</b> exitosa.", reconnection_count),
                            ).await;
                        }

                        // Crear suscripción a todas las vault accounts
                        let (sub_tx, sub_rx) = mpsc::channel(100);

                        let mut accounts_filter = std::collections::HashMap::new();
                        accounts_filter.insert(
                            "vault_monitor".to_string(),
                            crate::generated::geyser::SubscribeRequestFilterAccounts {
                                account: all_vault_addresses.clone(),
                                owner: vec![],
                                filters: vec![],
                            },
                        );

                        let request = crate::generated::geyser::SubscribeRequest {
                            accounts: accounts_filter,
                            slots: std::collections::HashMap::new(),
                            transactions: std::collections::HashMap::new(),
                            blocks: std::collections::HashMap::new(),
                            blocks_meta: std::collections::HashMap::new(),
                            entry: None,
                            commitment: Some(0), // PROCESSED — máxima velocidad
                            accounts_data_slice: std::collections::HashMap::new(),
                            ping: None,
                        };

                        if let Err(e) = sub_tx.send(request).await {
                            eprintln!("❌ [Geyser] Error enviando suscripción: {}", e);
                            tokio::time::sleep(reconnect_delay).await;
                            continue;
                        }

                        let stream_result = grpc_client
                            .subscribe(tokio_stream::wrappers::ReceiverStream::new(sub_rx))
                            .await;

                        match stream_result {
                            Ok(response) => {
                                let mut stream = response.into_inner();
                                println!(
                                    "📡 [Geyser] Stream activo — monitoreando {} vault accounts",
                                    all_vault_addresses.len()
                                );
                                let mut update_count: u64 = 0;
                                let mut last_data_at = Instant::now();

                                loop {
                                    // Watchdog: si no recibimos datos en `staleness_timeout`, 
                                    // asumimos stream zombie y forzamos reconexión
                                    let update = tokio::select! {
                                        msg = stream.message() => msg,
                                        _ = tokio::time::sleep(staleness_timeout) => {
                                            let stale_secs = last_data_at.elapsed().as_secs();
                                            eprintln!(
                                                "🧊 [Geyser] Stream ZOMBIE detectado — sin datos hace {}s",
                                                stale_secs
                                            );
                                            let _ = notifier.send_connectivity_alert(
                                                "Geyser gRPC",
                                                false,
                                                &format!(
                                                    "🧊 <b>Stream zombie</b> — sin datos hace {}s\n\
                                                     Updates antes del corte: {}",
                                                    stale_secs, update_count
                                                ),
                                            ).await;
                                            break;
                                        }
                                    };

                                    match update {
                                        Ok(Some(update)) => {
                                            last_data_at = Instant::now();
                                            if let Some(event) = update.update_oneof {
                                                match event {
                                                    crate::generated::geyser::subscribe_update::UpdateOneof::Account(acc) => {
                                            if let Some(info) = acc.account {
                                                let vault_address = bs58::encode(&info.pubkey).into_string();

                                                // ¿Esta vault pertenece a algún par que monitoreamos?
                                                if let Some(token_mint) = vault_to_mint.get(&vault_address) {
                                                    // Parsear el amount del SPL Token Account
                                                    if let Some(amount) = parse_spl_token_account_amount(&info.data) {
                                                        update_count += 1;

                                                        // Actualizar reserve en el tracker
                                                        let price_update = {
                                                            let mut tracker = vault_tracker.write().await;
                                                            if let Some(pair) = tracker.get_mut(token_mint) {
                                                                pair.update_reserve(&vault_address, amount);

                                                                // Intentar calcular precio
                                                                if pair.is_ready() {
                                                                    pair.calculate_price_in_quote().map(|price_sol| {
                                                                        (
                                                                            pair.symbol.clone(),
                                                                            pair.token_mint.clone(),
                                                                            price_sol,
                                                                            pair.calculate_liquidity_in_quote().unwrap_or(0.0),
                                                                        )
                                                                    })
                                                                } else {
                                                                    None
                                                                }
                                                            } else {
                                                                None
                                                            }
                                                        };

                                                        if let Some((symbol, mint, price_sol, liq_sol)) = price_update {
                                                            // Obtener precio de SOL desde el caché
                                                            let sol_usd = *sol_price.read().await;

                                                            // Si no tenemos SOL price, intentar sacarlo del caché de DexScreener
                                                            let sol_usd = if sol_usd == 0.0 {
                                                                // Buscar un precio de SOL en el caché general
                                                                let c = cache.read().await;
                                                                // SOL price: buscamos en cualquier token que tenga price_native > 0
                                                                c.values()
                                                                    .find(|p| p.price_native > 0.0 && p.price_usd > 0.0)
                                                                    .map(|p| p.price_usd / p.price_native)
                                                                    .unwrap_or(0.0)
                                                            } else {
                                                                sol_usd
                                                            };

                                                            // Actualizar SOL tracker si obtuvimos un precio
                                                            if sol_usd > 0.0 {
                                                                *sol_price.write().await = sol_usd;
                                                            }

                                                            let price_usd = price_sol * sol_usd;
                                                            let liquidity_usd = liq_sol * sol_usd;

                                                            // Obtener datos adicionales del caché (volume, change)
                                                            let (volume_24h, price_change_24h) = {
                                                                let c = cache.read().await;
                                                                c.get(&mint)
                                                                    .map(|p| (p.volume_24h, p.price_change_24h))
                                                                    .unwrap_or((0.0, 0.0))
                                                            };

                                                            let geyser_update = PriceUpdate {
                                                                token_mint: mint.clone(),
                                                                symbol: symbol.clone(),
                                                                price_usd,
                                                                price_native: price_sol,
                                                                liquidity_usd,
                                                                volume_24h,
                                                                price_change_24h,
                                                                source: PriceSource::Geyser,
                                                                received_at: Instant::now(),
                                                            };

                                                            // Actualizar caché con el dato de Geyser
                                                            {
                                                                let mut c = cache.write().await;
                                                                c.insert(mint.clone(), geyser_update.clone());
                                                            }

                                                            if tx.try_send(geyser_update).is_err() {
                                                                // Buffer lleno, dato ya en caché
                                                            }

                                                            if update_count.is_multiple_of(50) {
                                                                println!(
                                                                    "⚡ [Geyser] #{} {} = {:.10} SOL (${:.8}) | Liq: {:.1} SOL",
                                                                    update_count, symbol, price_sol, price_usd, liq_sol
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        crate::generated::geyser::subscribe_update::UpdateOneof::Ping(_) => {
                                            // Heartbeat — conexión viva (reset watchdog)
                                            last_data_at = Instant::now();
                                        },
                                        _ => {} // Ignorar otros eventos
                                    }
                                    }
                                        }
                                        Ok(None) => {
                                            // Stream cerrado limpiamente
                                            break;
                                        }
                                        Err(e) => {
                                            eprintln!("❌ [Geyser] Error en stream: {}", e);
                                            break;
                                        }
                                    }
                                }

                                println!("⚠️  [Geyser] Stream cerrado por el servidor (recibidos {} updates)", update_count);
                                let _ = notifier.send_connectivity_alert(
                                    "Geyser gRPC",
                                    false,
                                    &format!(
                                        "Stream cerrado por el servidor.\n\
                                         <b>Updates recibidos:</b> {}\n\
                                         Reconectando con backoff...",
                                        update_count
                                    ),
                                ).await;
                            }
                            Err(e) => {
                                eprintln!("❌ [Geyser] Error iniciando stream: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ [Geyser] Error de conexión: {}", e);
                        let _ = notifier.send_connectivity_alert(
                            "Geyser gRPC",
                            false,
                            &format!("Error de conexión: <code>{}</code>", e),
                        ).await;
                    }
                }

                // Exponential backoff para reconexión
                reconnection_count += 1;
                eprintln!(
                    "🔄 [Geyser] Reconexión #{} en {:?}...",
                    reconnection_count, reconnect_delay
                );
                tokio::time::sleep(reconnect_delay).await;
                reconnect_delay = (reconnect_delay * 2).min(max_reconnect_delay);
            }
        }
    }
}
