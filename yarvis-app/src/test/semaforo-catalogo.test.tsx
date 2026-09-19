// TEST — Parser CSV del catálogo de códigos (formato dataset).
// Puro: sin backend, sin mocks.
import { describe, it, expect } from "vitest";
import { parsearCsvBarras } from "../front-admin/ventanas/admininventario/PanelCodigos";

describe("parsearCsvBarras", () => {
  it("parsea filas reales del dataset con header", () => {
    const csv = [
      "ean,nombre,marca,cantidad,unidad,categoria",
      "7501011101456,SABRITAS ORIGINAL,Sabritas,42,g,botana",
      "7501055304745,COCA COLA ORIGINAL,Coca-Cola,600,ml,refrescos",
    ].join("\n");
    const filas = parsearCsvBarras(csv);
    expect(filas).toHaveLength(2);
    expect(filas[0]).toEqual({
      ean: "7501011101456",
      nombre: "SABRITAS ORIGINAL",
      marca: "Sabritas",
      cantidad: 42,
      unidad: "g",
      categoria: "botana",
    });
  });

  it("tolera BOM, CRLF, líneas vacías y sin header", () => {
    const csv = "﻿7501011101456,SABRITAS ORIGINAL,Sabritas,42,g,botana\r\n\r\n7501055304745,COCA COLA ORIGINAL,,600,ml,";
    const filas = parsearCsvBarras(csv);
    expect(filas).toHaveLength(2);
    expect(filas[1].marca).toBeNull();
    expect(filas[1].categoria).toBeNull();
  });

  it("descarta filas sin ean/nombre o cantidad inválida", () => {
    const csv = [
      "ean,nombre,marca,cantidad,unidad,categoria",
      ",SIN EAN,Sabritas,42,g,botana",
      "7501011101456,,Sabritas,42,g,botana",
      "7501011101456,MALA CANTIDAD,Sabritas,xx,g,botana",
      "7501011101456,BUENA,Sabritas,1.5,l,abarrotes",
    ].join("\n");
    const filas = parsearCsvBarras(csv);
    expect(filas).toHaveLength(1);
    expect(filas[0].cantidad).toBe(1.5);
  });
});
