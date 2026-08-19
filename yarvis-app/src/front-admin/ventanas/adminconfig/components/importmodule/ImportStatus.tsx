import { useParserContext } from "../../../../../hooks/ParserContext";

const ImportStatus = () => {
  const { catalogParsed, iaTrained, ticketsParsed, ticketsGuardados, ticketsCount } = useParserContext();

  let label: string;
  let colorClass: string;
  let dotClass: string;
  if (!catalogParsed && !iaTrained && !ticketsParsed) {
    label = "Esperando datos";
    colorClass = "bg-neutral-100 text-neutral-400";
    dotClass = "bg-neutral-300";
  } else if (catalogParsed && !iaTrained) {
    label = "Esperando entrenamiento de IA";
    colorClass = "bg-orange-50 text-orange-500";
    dotClass = "bg-orange-400";
  } else if (iaTrained && !ticketsParsed) {
    label = "Esperando parseamiento de tickets";
    colorClass = "bg-yellow-50 text-yellow-600";
    dotClass = "bg-yellow-400";
  } else {
    const ticketLabel = ticketsGuardados === 1 ? "1 ticket" : `${ticketsGuardados} tickets`;
    const productoLabel = ticketsCount === 1 ? "1 producto" : `${ticketsCount} productos`;
    label = `${ticketLabel} · ${productoLabel} parseados`;
    colorClass = "bg-green-50 text-green-600";
    dotClass = "bg-green-500";
  }

  return (
    <div className={`mx-8 mt-6 px-5 py-3 rounded-2xl flex items-center gap-3 ${colorClass}`}>
      <div className={`w-2 h-2 rounded-full ${dotClass}`}></div>
      <span className="text-[9px] font-black uppercase tracking-widest">{label}</span>
    </div>
  );
};

export default ImportStatus;