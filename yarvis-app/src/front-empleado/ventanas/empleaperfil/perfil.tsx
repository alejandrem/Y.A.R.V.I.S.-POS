const perfilNav = {
  id: "perfil",
  label: "PERFIL",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
    </svg>
  ),
};

export default function Perfil() {
  return (
    <div>
      <h1>Empleados</h1>
    </div>
  );
}

export { perfilNav };
