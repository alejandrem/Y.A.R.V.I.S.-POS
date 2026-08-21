import { MorphIcon } from 'morphicons/react';
import { ICONO_DOLAR } from '../../../components/ui';

const AdminFinanzas = () => (
  <section className="mx-auto w-full max-w-6xl rounded-[2.5rem] border border-neutral-100 bg-white p-6 shadow-xl sm:p-10">
    <div className="py-8 text-center sm:py-12">
      <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-neutral-950 text-neutral-50">
        <MorphIcon icon={ICONO_DOLAR} size={28} strokeWidth={2.1} spring="smooth" reducedMotion="user" />
      </div>
      <p className="mt-6 text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Módulo en reconstrucción</p>
      <h3 className="mt-2 text-2xl font-black text-neutral-900">Control financiero</h3>
      <p className="mx-auto mt-3 max-w-lg text-sm leading-6 text-neutral-500">
        Finanzas se está reorganizando desde sus datos reales: ventas completadas, pagos, gastos, cortes de caja, alertas y predicciones.
        La nueva interfaz se definirá después de cerrar el contrato de datos y las reglas de cálculo.
      </p>
    </div>
  </section>
);

export default AdminFinanzas;
