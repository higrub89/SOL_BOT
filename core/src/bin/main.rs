use anyhow::Result;

/// 🏎️ Configuración de Prioridad de Competición
fn set_hft_priority() {
    #[cfg(target_os = "linux")]
    unsafe {
        let param = libc::sched_param { sched_priority: 99 };
        let result = libc::sched_setscheduler(0, libc::SCHED_FIFO, &param);
        if result == 0 {
            println!("🏎️  CHASSIS: Prioridad SCHED_FIFO (99) activada con éxito.");
        } else {
            eprintln!("⚠️  CHASSIS: No se pudo activar SCHED_FIFO. Comprobar CAP_SYS_NICE.");
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
