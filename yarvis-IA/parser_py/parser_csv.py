"""
Parser de catálogos en formato CSV (.csv, .tsv).
Detecta separadores automáticamente (, o ;).
"""
from core.utils import limpiar_producto


def _detectar_separador_csv(linea: str) -> str | None:
    """Detecta si una línea es CSV y retorna el separador (, o ;)."""
    comas = linea.count(',')
    puntos_coma = linea.count(';')
    
    if comas >= 1 and comas > puntos_coma:
        return ','
    elif puntos_coma >= 1 and puntos_coma > comas:
        return ';'
    return None


def parsear_csv(texto: str) -> list[dict]:
    """
    Parsea un catálogo en formato CSV.
    
    Formatos soportados:
        nombre,costo,venta,categoria,stock
        nombre;costo;venta;categoria;stock
        nombre,venta,costo  (detecta automáticamente el orden)
    """
    lineas = texto.strip().splitlines()
    if not lineas:
        return []
    
    # Detectar separador de la primera línea
    separador = _detectar_separador_csv(lineas[0])
    if not separador:
        return []
    
    # Detectar si la primera línea es header
    primera_linea = lineas[0].lower()
    tiene_header = any(p in primera_linea for p in ['nombre', 'producto', 'precio', 'costo', 'venta', 'categoria', 'stock', 'cantidad', 'existencia'])
    
    # Detectar headers si existen
    headers = []
    if tiene_header:
        headers = [h.strip().lower() for h in lineas[0].split(separador)]
    
    # Buscar índice de columna de stock/cantidad
    col_stock = None
    for i, h in enumerate(headers):
        if any(k in h for k in ['stock', 'existencia', 'cantidad', 'inventario', 'qty', 'quantity']):
            col_stock = i
            break
    
    # Buscar índice de columna de categoría
    col_categoria = None
    for i, h in enumerate(headers):
        if any(k in h for k in ['categoría', 'categoria', 'tipo', 'seccion', 'sección', 'departamento', 'category', 'type']):
            col_categoria = i
            break
    
    productos = []
    
    for i, linea in enumerate(lineas):
        linea = linea.strip()
        if not linea:
            continue
        
        # Saltar header si existe
        if tiene_header and i == 0:
            continue
        
        partes = [p.strip().strip('"').strip("'") for p in linea.split(separador)]
        
        # Necesitamos al menos 2 columnas (nombre + precio)
        if len(partes) < 2:
            continue
        
        # Intentar detectar el orden de columnas
        nombre = None
        precio_venta = None
        precio_costo = None
        categoria = None
        stock = 0
        
        # Buscar columnas numéricas
        numeric_cols = []
        text_cols = []
        for j, p in enumerate(partes):
            p_clean = p.replace('$', '').replace(',', '').strip()
            try:
                val = float(p_clean)
                numeric_cols.append((j, val))
            except ValueError:
                if p_clean:
                    text_cols.append((j, p_clean))
        
        # La primera columna de texto es el nombre
        for j, p in text_cols:
            if not p.replace('.', '').isdigit():
                nombre = p
                break
        
        if not nombre:
            continue
        
        # Si solo hay 2 columnas de texto y no hay precios, 
        # la primera puede ser categoría y la segunda producto
        if len(numeric_cols) == 0 and len(text_cols) == 2:
            first_text = text_cols[0][1]
            second_text = text_cols[1][1]
            # El nombre más largo probablemente es el producto
            if len(second_text) > len(first_text):
                nombre = second_text
                categoria = first_text.upper()
            else:
                nombre = first_text
                categoria = second_text.upper()
        
        # Asignar precios
        if len(numeric_cols) >= 2:
            idx1, val1 = numeric_cols[0]
            idx2, val2 = numeric_cols[1]
            
            if val1 > val2:
                precio_venta = val1
                precio_costo = val2
            else:
                precio_venta = val2
                precio_costo = val1
            
            # Buscar categoría en la última columna de texto
            for j, p in reversed(text_cols):
                if p != nombre and len(p) < 30:
                    categoria = p.upper()
                    break
        elif len(numeric_cols) == 1:
            _, val = numeric_cols[0]
            precio_venta = val
            precio_costo = 0
        
        # Obtener stock de la columna detectada
        if col_stock is not None and col_stock < len(partes):
            stock_str = partes[col_stock].replace(',', '').strip()
            try:
                stock = int(float(stock_str))
            except (ValueError, TypeError):
                stock = 0
        
        # Obtener categoría de la columna detectada
        if col_categoria is not None and col_categoria < len(partes) and partes[col_categoria]:
            categoria = partes[col_categoria].strip().upper()
        
        # Si no hay categoría aún, buscar en columnas de texto que no sean el nombre
        if categoria == "SIN CATEGORÍA" or categoria is None:
            for j, p in text_cols:
                if p != nombre and len(p) < 30:
                    categoria = p.upper()
                    break
        
        # Si solo hay 2 columnas de texto, la primera puede ser categoría
        if (categoria == "SIN CATEGORÍA" or categoria is None) and len(text_cols) == 2:
            first_text = text_cols[0][1]
            second_text = text_cols[1][1]
            if first_text != nombre:
                categoria = first_text.upper()
            elif second_text != nombre:
                categoria = second_text.upper()
        
        if precio_venta and precio_venta > 0:
            productos.append({
                "nombre": limpiar_producto(nombre),
                "precio_costo": round(precio_costo or 0, 2),
                "precio_venta": round(precio_venta, 2),
                "stock": stock,
                "categoria": categoria or "SIN CATEGORÍA",
            })
        elif nombre:
            # Sin precios, agregar con precio 0
            productos.append({
                "nombre": limpiar_producto(nombre),
                "precio_costo": 0,
                "precio_venta": 0,
                "stock": stock,
                "categoria": categoria or "SIN CATEGORÍA",
            })
    
    return productos
