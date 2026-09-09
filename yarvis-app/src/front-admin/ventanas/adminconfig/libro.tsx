// Wrapper delgado del Libro para el ADMIN.
// Mismo patron que inventario.tsx / yarvis.tsx: el panel canonico vive en
// un solo lugar (empleaajustes/datos inutiles/Libro) y cada rol lo usa con
// su prop `rol`. Aqui solo fijamos rol="admin" (portada dice
// "MANUAL DEL ADMINISTRADOR"); el contenido es el mismo manual.
import Libro from "../../../front-empleado/ventanas/empleaajustes/datos inutiles/Libro";

const LibroAdmin = () => <Libro rol="admin" />;

export default LibroAdmin;
