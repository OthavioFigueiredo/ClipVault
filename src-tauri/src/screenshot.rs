use crate::storage::{StorageManager, VaultRecord};
use arboard::{Clipboard, ImageData};
use image::{ImageBuffer, Rgba};
use std::borrow::Cow;
use std::sync::Arc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, ImageFormat, Window};

/// Converte o buffer de pixels ZPixmap do X11 (normalmente BGRA/BGR0 em Little Endian) para RgbaImage
fn x11_zpixmap_to_rgba(data: &[u8], width: u32, height: u32, depth: u8) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    if depth != 24 && depth != 32 {
        return Err(format!("Profundidade de cor não suportada: {}", depth));
    }

    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);

    for chunk in data.chunks_exact(4) {
        let b = chunk[0];
        let g = chunk[1];
        let r = chunk[2];
        let a = 255u8;
        rgba_bytes.extend_from_slice(&[r, g, b, a]);
    }

    ImageBuffer::from_raw(width, height, rgba_bytes)
        .ok_or_else(|| "Falha ao construir buffer de imagem RGBA".to_string())
}

/// Captura a tela inteira utilizando protocolo nativo X11
pub fn capture_fullscreen(storage: &Arc<StorageManager>) -> Result<VaultRecord, String> {
    let (conn, screen_num) = x11rb::connect(None)
        .map_err(|e| format!("Falha ao conectar ao servidor X11: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let geom = conn.get_geometry(root)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;

    let width = geom.width as u32;
    let height = geom.height as u32;

    let img_reply = conn.get_image(
        ImageFormat::Z_PIXMAP,
        root,
        0,
        0,
        geom.width,
        geom.height,
        !0,
    )
    .map_err(|e| format!("Falha na chamada get_image do X11: {}", e))?
    .reply()
    .map_err(|e| format!("Falha na resposta get_image do X11: {}", e))?;

    let rgba_img = x11_zpixmap_to_rgba(&img_reply.data, width, height, img_reply.depth)?;

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    rgba_img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Falha ao codificar PNG: {}", e))?;

    // Salvar no histórico
    let record = storage.add_image_record(png_bytes.clone(), width, height)?
        .ok_or_else(|| "Falha ao salvar registro da captura".to_string())?;

    // Copiar automaticamente para o clipboard
    copy_rgba_to_clipboard(&rgba_img, width, height);

    Ok(record)
}

/// Captura a janela ativa
pub fn capture_active_window(storage: &Arc<StorageManager>) -> Result<VaultRecord, String> {
    let (conn, screen_num) = x11rb::connect(None)
        .map_err(|e| format!("Falha ao conectar ao servidor X11: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?
        .atom;

    let active_win_reply = conn.get_property(
        false,
        root,
        net_active_window,
        AtomEnum::WINDOW,
        0,
        1,
    )
    .map_err(|e| e.to_string())?
    .reply()
    .map_err(|e| e.to_string())?;

    let active_window: Window = match active_win_reply.value.get(..4) {
        Some(slice) => u32::from_ne_bytes(slice.try_into().unwrap_or([0, 0, 0, 0])),
        None => return Err("Nenhuma janela ativa detectada".to_string()),
    };

    if active_window == 0 {
        return Err("ID da janela ativa inválido".to_string());
    }

    // Obter geometria da janela ativa
    let geom = conn.get_geometry(active_window)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;

    // Traduzir coordenadas relativas para o root para capturar inclusive decorações/composição
    let coords = conn.translate_coordinates(active_window, root, 0, 0)
        .map_err(|e| e.to_string())?
        .reply()
        .map_err(|e| e.to_string())?;

    let width = geom.width as u32;
    let height = geom.height as u32;

    let img_reply = conn.get_image(
        ImageFormat::Z_PIXMAP,
        root,
        coords.dst_x,
        coords.dst_y,
        geom.width,
        geom.height,
        !0,
    )
    .map_err(|e| format!("Falha na chamada get_image da janela: {}", e))?
    .reply()
    .map_err(|e| format!("Falha na resposta get_image da janela: {}", e))?;

    let rgba_img = x11_zpixmap_to_rgba(&img_reply.data, width, height, img_reply.depth)?;

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    rgba_img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Falha ao codificar PNG da janela: {}", e))?;

    let record = storage.add_image_record(png_bytes, width, height)?
        .ok_or_else(|| "Falha ao salvar registro da captura".to_string())?;

    copy_rgba_to_clipboard(&rgba_img, width, height);

    Ok(record)
}

/// Captura uma região selecionada (coordenadas no root)
pub fn capture_region(
    storage: &Arc<StorageManager>,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
) -> Result<VaultRecord, String> {
    if width == 0 || height == 0 {
        return Err("Dimensões da região inválidas (largura ou altura zero)".to_string());
    }

    let (conn, screen_num) = x11rb::connect(None)
        .map_err(|e| format!("Falha ao conectar ao servidor X11: {}", e))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    let img_reply = conn.get_image(
        ImageFormat::Z_PIXMAP,
        root,
        x,
        y,
        width,
        height,
        !0,
    )
    .map_err(|e| format!("Falha na captura da região X11: {}", e))?
    .reply()
    .map_err(|e| format!("Falha na resposta da região: {}", e))?;

    let w32 = width as u32;
    let h32 = height as u32;
    let rgba_img = x11_zpixmap_to_rgba(&img_reply.data, w32, h32, img_reply.depth)?;

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    rgba_img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Falha ao codificar PNG da região: {}", e))?;

    let record = storage.add_image_record(png_bytes, w32, h32)?
        .ok_or_else(|| "Falha ao salvar registro da região".to_string())?;

    copy_rgba_to_clipboard(&rgba_img, w32, h32);

    Ok(record)
}

/// Copia buffer RGBA para a área de transferência
fn copy_rgba_to_clipboard(rgba_img: &ImageBuffer<Rgba<u8>, Vec<u8>>, width: u32, height: u32) {
    let raw = rgba_img.clone().into_raw();
    if let Ok(mut cb) = Clipboard::new() {
        let _ = cb.set_image(ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(raw),
        });
    }
}
