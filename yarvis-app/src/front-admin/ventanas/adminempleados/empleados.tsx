const AdminEmpleados = () => (
  <div className="max-w-3xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto">
    <header className="mb-10 text-center">
      <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">EMPLEADOS</h2>
      <div className="h-1.5 w-12 bg-neutral-900 rounded-full mx-auto"></div>
    </header>
    <div className="bg-neutral-50 p-10 rounded-[2.5rem] border border-neutral-100 shadow-inner">
      <p className="text-xl text-neutral-600 font-light leading-relaxed first-letter:text-4xl first-letter:font-black first-letter:text-neutral-900">
        Desde aqui podremos gestionar quienes son los empleados nombre de los empleados, cuanto venden, ventas canceladas, con descuentos ventas, metas que les puedes poner para generar bonos, definir turnos, horarios, definir su salario semanal, para que puedas visualizar cuanto te estan generando y cuanto te esta costando mantenerlo, visualizar cuando hicieron cortes de caja x y z recordemos que el corte de caja Z es el cierre de turno, visualizar cuando iniciaron turno
      </p>
    </div>
  </div>
);

export default AdminEmpleados;
