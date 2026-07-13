import ChatWidget from "../../../components/ChatWidget";

const AdminYarvis = () => (
  <div className="h-full animate-in fade-in duration-500">
    <ChatWidget
      role="admin"
      userId="admin"
      suggestions={[
        "¿Hubo algo raro hoy?",
        "¿Cuánto gané libre hoy quitando el costo de los productos?",
        "¿Qué debería comprar para el fin de semana?",
        "¿Qué productos están por agotarse?",
        "Resumen de ventas de hoy",
        "¿Qué empleados tienen más reembolsos?",
      ]}
    />
  </div>
);

export default AdminYarvis;
