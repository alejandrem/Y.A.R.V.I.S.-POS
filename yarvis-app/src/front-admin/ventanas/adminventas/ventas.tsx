// Módulo Ventas (en desarrollo).
// Contenido provisional: aviso de que está en producción próximamente.
// Reemplazará el placeholder de texto sin UI.
const AdminVentas = () => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
    <div className="text-center py-8 sm:py-12">
      <div className="mx-auto w-16 h-16 rounded-2xl bg-neutral-950 text-neutral-50 flex items-center justify-center">
        <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" /></svg>
      </div>
      <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400 mt-6">Módulo en producción</p>
      <h3 className="text-2xl font-black text-neutral-900 mt-2">Ventas y predicciones</h3>
      <p className="text-sm text-neutral-500 mt-3 max-w-md mx-auto">Próximamente podrás ver cuánto vas a vender el próximo fin de semana o mes con intervalos de confianza, aprendiendo de tus ventas reales.</p>
    </div>
  </section>
);

export default AdminVentas;