# si te sale error por version del compilador de rust es por que hay un bug al intentar compilar 

## version windows

la solucion es cambiar a una version estable de rust ejecuta esto en tu terminal 


1- actualiza el gestor de rust y asegura la version estable 
```
rustup update stable
```

2- establece una version estable como la predeterminada:
```
rustup default stable
``` 
3- limpia los archivos temporales de compilacion previos
```
cd yarvis-app/src-tauri
cargo clean
cd 
```
4-  volver a la carpeta inicial del script
```
.\run.bat
```



## version linux

5- o si usas linux usa esta serie de pasos
```
cd yarvis-app/src-tauri
cargo clean
cd .. 
./run.sh

```