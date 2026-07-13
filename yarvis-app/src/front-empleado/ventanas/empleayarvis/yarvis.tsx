import ChatWidget from "../../../components/ChatWidget";

const yarvisNav = {
  id: "yarvis",
  label: "Y.A.R.V.I.S.",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 8V4H8" />
      <rect width="16" height="12" x="4" y="8" rx="2" />
      <path d="M2 14h2" />
      <path d="M20 14h2" />
      <path d="M15 13v2" />
      <path d="M9 13v2" />
    </svg>
  ),
};

export default function Yarvis() {
  return (
    <div className="h-full animate-in fade-in duration-500">
      <ChatWidget
        role="empleado"
        userId="empleado"
        suggestions={[
          "¿Qué productos tengo de limpieza?",
          "¿Cuánto stock hay de Coca-Cola?",
          "¿Qué es lo más vendido esta semana?",
          "¿Qué productos no tienen sal?",
        ]}
      />
    </div>
  );
}

export { yarvisNav };
