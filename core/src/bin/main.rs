use anyhow::Result;

/// 🏎️ Configuración de Prioridad de Competición
/// Requiere CAP_SYS_NICE capability en Linux para funcionar.
/// Si falla, el proceso continúa con prioridad normal.
fn set_hft_priority() {
    #[cfg(target_os = "linux")]
    unsafe {
        let param = libc::sched_param { sched_priority: 99 };
        let result = libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
        if result == 0 {
            println!("🏎️  CHASSIS: Prioridad SCHED_FIFO (99) activada con éxito.");
        } else {
            eprintln!("⚠️  CHASSIS: No se pudo activar SCHED_FIFO ({}). Continuando con prioridad normal.", std::io::Error::last_os_error());
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Reclamar el hardware antes de arrancar
    set_hft_priority();

    // 2. Ejecutar el motor
    the_chassis::run().await
}
