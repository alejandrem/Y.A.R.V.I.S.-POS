# Dataset de Códigos de Barras (REALES)

## Estructura

```
dataset_barras/
├── refrescos.csv
├── botana.csv
├── lacteos.csv
├── abarrotes.csv
└── README.md
```

## Reglas de captura

- **Formato**: CSV, UTF-8, un archivo por categoría.
- **Header**: `ean,nombre,marca,cantidad,unidad,categoria`
- **Sin ID**: el `ean` es la llave.

## Columnas

| Columna | Descripción | Ejemplo |
|---------|-------------|---------|
| `ean` | Código de barras EAN-13 o UPC (texto, 12-13 dígitos). Llave única. | `7501011101456` |
| `nombre` | Nombre del producto en MAYÚSCULAS, sin presentación. | `SABRITAS ORIGINAL` |
| `marca` | Marca comercial. | `Sabritas` |
| `cantidad` | Cantidad numérica. | `42` |
| `unidad` | Unidad de catálogo cerrado: `ml` \| `l` \| `g` \| `kg` \| `pzs`. | `g` |
| `categoria` | Categoría del producto (igual al nombre del archivo sin `.csv`). | `botana` |

## Validaciones

1. **EAN único**: si un código aparece dos veces con distinto producto, se marca conflicto.
2. **Dígito verificador**: todo EAN-13 debe tener check digit válido.
3. **Unidades**: solo `ml`, `l`, `g`, `kg`, `pzs`. Nada de `gr`, `ML`, `litros`.
4. **Nombre**: MAYÚSCULAS, sin acentos ni comas.
5. **Sin presentación en nombre**: la presentación va en `cantidad` + `unidad`.

## Ejemplo

```csv
ean,nombre,marca,cantidad,unidad,categoria
7501011101456,SABRITAS ORIGINAL,Sabritas,42,g,botana
7501011101463,SABRITAS ADOBADA,Sabritas,42,g,botana
7501055304745,COCA COLA ORIGINAL,Coca-Cola,600,ml,refrescos
7501031310012,PEPSI ORIGINAL,Pepsi,600,ml,refrescos
```

## Productos por archivo

- `refrescos.csv`: 128 productos (Coca-Cola, Pepsi, Gatorade, Fanta, Sprite, Sidral, Squirt, Fresca, Dr Pepper, Joya, Hidrolit, Lipton, Jumex, Epurita, Powerade, Ciel, Electrolit, Suerox, Flashlyte, Volt, Red Bull, Vive, Amper, Vitaloe, Predator, Arizona, Del Valle, Fuze Tea, Vitamin Water, Ocean Spray, etc.)
- `botana.csv`: 45 productos (Sabritas, Ruffles, Doritos, Cheetos, Tostitos, Fritos, Paketaxo, Sabritones, Crujitos, Rancheritos, Churrumais, Bolzaza, Cacahuates, Barcel, Mission, Bitz)
- `lacteos.csv`: 15 productos (Santa Clara, Alpura, Lala, Nutralat, Chocomilk)
- `abarrotes.csv`: 120 productos (Bimbo, Marinela, Kellogg's, Hershey's, Oreo, Ritz, Bitz, La Posada, Nutrioli, Bachoco, Selecta, Dolores, Nestle, Hunt's, Del Monte, Heinz, Tuny, Maruchan, Tikytin, Azalea, Colgate, Zest, KBB, Saba, Always, Palmolive, Stefano, Suavitel, Axion, Cloralex, Salvo, Downy, Ensueno, Pinol, Purina, Ganador, etc.)

**Total**: 308 productos

## Fuentes

Códigos extraídos de documentos reales de planogramas y listas de precios de:
- Promoxxo (planogramas de tiendas)
- Órdenes de compra Sabritas
- Planogramas de refrescos

## Notas

- Los EANs son **códigos reales** usados actualmente en productos de México.
- Prefijos: 750 (México), 7501011 (Sabritas/PepsiCo), 7501031 (Pepsi), 7501055 (Coca-Cola FEMSA), etc.
- Algunos códigos UPC (12 dígitos) también incluidos (ej. 661440000014, 815154022705).
- Para usar en producción, validar contra empaques reales y ajustar nombres/cantidades según corresponda.