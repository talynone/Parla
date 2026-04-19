// Hook : copie du texte dans le clipboard avec feedback visuel transitoire.
//
// Reference VoiceInk Views/Components/CopyIconButton.swift : icone qui
// passe de "doc.on.doc" a "checkmark" pendant 1.5s apres copie.

import { useCallback, useRef, useState } from "react";

export function useCopyButton(durationMs = 1500) {
  const [copied, setCopied] = useState(false);
  const timer = useRef<number | null>(null);

  const copy = useCallback(
    async (text: string | null | undefined) => {
      if (!text) return;
      try {
        await navigator.clipboard.writeText(text);
        if (timer.current !== null) {
          window.clearTimeout(timer.current);
        }
        setCopied(true);
        timer.current = window.setTimeout(() => {
          setCopied(false);
          timer.current = null;
        }, durationMs);
      } catch (e) {
        console.error("clipboard write failed", e);
      }
    },
    [durationMs],
  );

  return { copied, copy };
}
