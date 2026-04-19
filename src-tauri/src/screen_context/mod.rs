// Module screen_context : capture de la fenetre active + OCR pour remplir
// le bloc <CURRENT_WINDOW_CONTEXT> de l'enhancement.
//
// Reference VoiceInk : Services/ScreenCaptureService.swift + l'assemblage
// dans AIEnhancementService.getSystemMessage.

pub mod capture;
pub mod ocr;
pub mod service;
