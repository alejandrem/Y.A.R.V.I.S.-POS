const AdminVentas = () => (
  <div className="max-w-3xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto">
    <header className="mb-10 text-center">
      <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">VENTAS</h2>
      <div className="h-1.5 w-12 bg-neutral-900 rounded-full mx-auto"></div>
    </header>
    <div className="bg-neutral-50 p-10 rounded-[2.5rem] border border-neutral-100 shadow-inner">
      <p className="text-xl text-neutral-600 font-light leading-relaxed first-letter:text-4xl first-letter:font-black first-letter:text-neutral-900">
        El sistema debe aprender de tus ventas y decirte cuanto vas a vender el proximo fin de semana, mes etc. con un margen de error, el sistema presente las predicciones con Intervalos de Confianza seran graficas complejas pero tambien
      </p>
    </div>
  </div>
);

export default AdminVentas;
