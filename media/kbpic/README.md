# Community gallery — how to add your build

> 🇪🇸 **¿Prefieres leer esto en español?** → [README.es.md](README.es.md)

Thanks for wanting to share your BFS9000 build! Here's how:

1. Create a folder with your name (or handle) under `media/kbpic/`:

   ```
   media/kbpic/<your-name>/
   ```

2. Add your photos inside it (JPG or PNG, ideally a few MB or less each). Name them however you like, for example `01.jpg`, `02.jpg`...

3. Copy the [`_template/README.md`](_template/README.md) file into your folder and fill it in with your photos and a short note about your build.

4. Add your build to the full gallery, [`media/GALLERY.md`](../GALLERY.md) (or [`media/GALLERY.es.md`](../GALLERY.es.md), if you'd rather write your caption in Spanish), using this template:

   ```markdown
   ## Your Name

   <p align="center">
     <img src="kbpic/<your-name>/01.jpg" width="32%">
     <img src="kbpic/<your-name>/02.jpg" width="32%">
     <img src="kbpic/<your-name>/03.jpg" width="32%">
   </p>

   [See all photos →](kbpic/<your-name>/README.md)
   ```

   Replace the folder name, image paths and your name. If you'd like, add a link to your GitHub or socials next to your name too.

5. If you'd like your build to also appear as a highlight on the main [`README.md`](../../README.md), add a small entry to the **Community builds** teaser there as well — keep it to a photo or two, the full set belongs in the gallery.

6. Open a pull request with your new photos and the README changes.

That's it — thanks for contributing!
