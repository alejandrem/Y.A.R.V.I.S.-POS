const AdminClientes = () => (
  <div className="max-w-3xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto">
    <header className="mb-10 text-center">
      <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">CLIENTES</h2>
      <div className="h-1.5 w-12 bg-neutral-900 rounded-full mx-auto"></div>
    </header>
    <div className="bg-neutral-50 p-10 rounded-[2.5rem] border border-neutral-100 shadow-inner">
      <p className="text-xl text-neutral-600 font-light leading-relaxed first-letter:text-4xl first-letter:font-black first-letter:text-neutral-900">
        Aqui mostraremos la gestion de los clientes y pedidos como Quien me compra, cuanto me compra, cuanto me genera y cuando fue su ultima visita, ademas de ver los pedidos encargados por clientes generados por el empleado, visualizando el formato de pago, los productos, si se aplico algun descuento, los tipos de ganancia etc. con la opcion de crear facturas reportes
      </p>
    </div>
  </div>
);

export default AdminClientes;
