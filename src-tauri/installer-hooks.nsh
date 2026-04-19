; NSIS installer hooks for Parla.
;
; Referenced from tauri.conf.json bundle.windows.nsis.installerHooks.
; Tauri's NSIS template calls the four macros below at well-defined
; stages of install / uninstall. We use the post-uninstall hook to wipe
; every piece of state Parla has created on disk + a few registry keys,
; so a reinstall starts fresh (triggers the onboarding, forgets which
; models were downloaded, clears any API key reference).
;
; Data Parla writes :
;   %APPDATA%\com.litterabbit.parla\
;     parla.settings.json          store plugin - general settings + onboarding flag
;     parla.prompts.json           store plugin - custom prompts + active prompt
;     parla.power_mode.json        store plugin - Power Mode profiles
;   %LOCALAPPDATA%\com.litterabbit.parla\
;     Models\                      Whisper .bin files
;     ParakeetModels\              Parakeet ONNX files
;     LlmModels\                   llama.cpp GGUF files
;     VAD\                         Silero VAD ONNX
;     Recordings\                  WAV audio captures
;     history.sqlite3              history DB
;     logs\                        tracing output
;   HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Parla
;     autostart registration (if user enabled it)
;
; Credential Manager (API keys) is NOT cleared - keyring-rs stores each
; provider as a separate target and there is no reliable enumeration from
; NSIS. They remain orphaned until the user revisits Credential Manager.
; A future enhancement could add a small PowerShell block here that loops
; the known provider ids (anthropic, openai, gemini, groq, cerebras,
; mistral, openrouter, deepgram, elevenlabs, soniox, speechmatics) and
; calls `cmdkey /delete:<id>` for each.

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTINSTALL
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Silent uninstalls (auto-updater flow : new installer runs old
  ; uninstaller with /S before installing the new version) must NOT
  ; wipe user data. Otherwise every auto-update would destroy models
  ; and settings. Only interactive uninstalls (Control Panel, Settings
  ; Apps) get to prompt the user.
  IfSilent parla_skip_wipe 0

    ; Ask the user whether to remove all Parla data (settings, models,
    ; history, recordings, autostart). Default is "No" so clicking
    ; through preserves data by accident. /SD IDNO also makes silent
    ; runs answer No, which matches the IfSilent guard above anyway.
    MessageBox MB_YESNO|MB_ICONQUESTION \
      "Do you also want to delete all Parla user data?$\r$\n$\r$\nThis will remove:$\r$\n    - Settings and onboarding state$\r$\n    - Custom prompts and Power Mode profiles$\r$\n    - Transcription history$\r$\n    - Downloaded models (Whisper, Parakeet, llama.cpp, VAD)$\r$\n    - Cached recordings and logs$\r$\n    - Autostart entry$\r$\n$\r$\nChoose No to keep them for a future reinstall.$\r$\n$\r$\nNote: API keys stored in Windows Credential Manager are never removed automatically. You can clear them from Control Panel > Credential Manager if desired." \
      /SD IDNO IDNO parla_skip_wipe

    ; User stores (tauri-plugin-store JSON files)
    RMDir /r "$APPDATA\com.litterabbit.parla"

    ; Models, history DB, recordings, logs
    RMDir /r "$LOCALAPPDATA\com.litterabbit.parla"

    ; Autostart entry (if enabled via Settings > General)
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Parla"

  parla_skip_wipe:
!macroend
