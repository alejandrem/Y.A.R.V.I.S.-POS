// Módulo Cortes de caja (en desarrollo).
// Contenido provisional: aviso de que está en producción próximamente.
const Cortes = () => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
    <div className="text-center py-8 sm:py-12">
      <div className="mx-auto w-16 h-16 rounded-2xl bg-neutral-950 text-neutral-50 flex items-center justify-center">
        <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
      </div>
      <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400 mt-6">Módulo en producción</p>
      <h3 className="text-2xl font-black text-neutral-900 mt-2">Cortes de caja</h3>
      <p className="text-sm text-neutral-500 mt-3 max-w-md mx-auto">Próximamente podrás registrar, revisar y cerrar cortes de caja desde este módulo. Mientras tanto, seguí usando el flujo de tickets.</p>
    </div>
  </section>
);

export default Cortes;