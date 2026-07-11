# Galería de la comunidad — cómo añadir tu montaje

> 🇬🇧 **Prefer to read this in English?** → [README.md](README.md)

¡Gracias por querer compartir tu montaje de BFS9000! Así es como se hace:

1. Crea una carpeta con tu nombre (o alias) dentro de `media/kbpic/`:

   ```
   media/kbpic/<tu-nombre>/
   ```

2. Añade tus fotos dentro de esa carpeta (JPG o PNG, a ser posible de pocos MB cada una). Puedes nombrarlas como quieras, por ejemplo `01.jpg`, `02.jpg`...

3. Copia el archivo [`_template/README.es.md`](_template/README.es.md) dentro de tu carpeta y rellénalo con tus fotos y una nota breve sobre tu montaje.

4. Añade tu montaje a la galería completa, [`media/GALLERY.es.md`](../GALLERY.es.md) (o [`media/GALLERY.md`](../GALLERY.md), si prefieres escribir el pie de foto en inglés), usando esta plantilla:

   ```markdown
   ## Tu Nombre

   <p align="center">
     <img src="kbpic/<tu-nombre>/01.jpg" width="32%">
     <img src="kbpic/<tu-nombre>/02.jpg" width="32%">
     <img src="kbpic/<tu-nombre>/03.jpg" width="32%">
   </p>

   [Ver todas las fotos →](kbpic/<tu-nombre>/README.es.md)
   ```

   Cambia el nombre de la carpeta, las rutas de las imágenes y tu nombre. Si quieres, añade también un enlace a tu GitHub o redes junto a tu nombre.

5. Si además quieres que tu montaje aparezca como destacado en el [`README.es.md`](../../README.es.md) principal, añade también una pequeña entrada en el apartado **Galería de la comunidad** de ahí — con una foto o dos es suficiente, el conjunto completo va en la galería.

6. Abre una pull request con tus fotos nuevas y los cambios en el README.

¡Eso es todo — gracias por participar!
