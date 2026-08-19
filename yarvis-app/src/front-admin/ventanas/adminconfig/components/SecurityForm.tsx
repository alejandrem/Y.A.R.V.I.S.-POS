import { PASS_PLACEHOLDER } from "../hooks/useAdminData";

interface SecurityFormProps {
  currentPass: string;
  setCurrentPass: (v: string) => void;
  passwordChanged: boolean;
  setPasswordChanged: (v: boolean) => void;
  onSave: () => void;
}

const SecurityForm = ({
  currentPass,
  setCurrentPass,
  passwordChanged,
  setPasswordChanged,
  onSave,
}: SecurityFormProps) => {
  return (
    <div className="bg-neutral-900 p-8 rounded-[2.5rem] shadow-2xl text-white space-y-6 relative overflow-hidden group">
      <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 rounded-full -translate-y-16 translate-x-16 blur-3xl group-hover:bg-white/10 transition-all"></div>
      <h3 className="text-[10px] font-black text-neutral-500 uppercase tracking-[0.4em]">Seguridad & Acceso</h3>
      <div className="space-y-4 relative z-10">
        <div className="group/input">
          <label className="text-[9px] font-black uppercase ml-2 mb-1 flex items-center gap-2 transition-colors">
            <span className={passwordChanged ? "text-yellow-400" : "text-neutral-500"}>Contraseña Maestra</span>
            {passwordChanged ? (
              <span className="text-[8px] text-yellow-400 font-black tracking-widest">● EDITANDO</span>
            ) : (
              <span className="text-[8px] text-green-400 font-black tracking-widest">✓ GUARDADA</span>
            )}
          </label>
          <input
            type="password"
            value={currentPass}
            onFocus={() => {
              if (!passwordChanged) {
                setCurrentPass("");
              }
            }}
            onBlur={() => {
              if (!passwordChanged && currentPass === "") {
                setCurrentPass(PASS_PLACEHOLDER);
              }
            }}
            onChange={(e) => {
              setCurrentPass(e.target.value);
              setPasswordChanged(true);
            }}
            className="w-full bg-white/5 border border-white/10 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-white/5 focus:border-white/20 transition-all text-white placeholder:text-white/20"
            placeholder="Nueva contraseña..."
          />
        </div>
        <button
          onClick={onSave}
          className="w-full bg-white text-neutral-900 py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all shadow-xl shadow-white/5"
        >
          Guardar Cambios
        </button>
      </div>
    </div>
  );
};

export default SecurityForm;