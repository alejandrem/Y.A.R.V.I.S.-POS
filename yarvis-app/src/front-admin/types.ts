export interface ColumnMapping {
  cantidad: number | null;
  producto: number[] | null;
  precio_unitario: number | null;
  total: number | null;
  descuento: number | null;
}

export interface LLMAnalysis {
  status: string;
  mapeo: {
    formato_detectado: string;
    columnas: ColumnMapping;
    delimitador: string;
    moneda: string;
    total_columnas: number;
    tiene_descuento: boolean;
    tiene_iva: boolean;
  };
  ejemplo_parseado: any[];
  confianza: number;
  notas: string;
  reintentado_con: string | null;
}
