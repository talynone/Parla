// View-switcher for the three transcription source kinds.
//
// IMPORTANT : clicking a tile only changes which panel is shown below
// (via `onViewKindChange`). It does NOT change the active kind in the
// store - that only happens when the user explicitly activates a model
// in the sub-panel (ParakeetPanel "Activer", ModelsPanel star, or a
// cloud model click in CloudProvidersPanel).
//
// The tile currently highlighted = `viewKind`. The tile with an "actif"
// badge = `activeKind` (the one the pipeline will use at record time).

import { useTranslation } from "react-i18next";
import { Cloud, Cpu, Zap, Check } from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";

type Kind = "local" | "cloud" | "parakeet";

export function TranscriptionSourcePanel({
  viewKind,
  activeKind,
  onViewKindChange,
}: {
  viewKind: Kind;
  activeKind: Kind;
  onViewKindChange: (kind: Kind) => void;
}) {
  const { t } = useTranslation();
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">{t("transcriptionSource.title")}</CardTitle>
        <CardDescription>{t("transcriptionSource.description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-3">
          <SourceTile
            selected={viewKind === "local"}
            active={activeKind === "local"}
            icon={<Cpu className="h-5 w-5" />}
            label={t("transcriptionSource.whisperLocalLabel")}
            description={t("transcriptionSource.whisperLocalDescription")}
            onClick={() => onViewKindChange("local")}
          />
          <SourceTile
            selected={viewKind === "parakeet"}
            active={activeKind === "parakeet"}
            icon={<Zap className="h-5 w-5" />}
            label={t("transcriptionSource.parakeetLabel")}
            description={t("transcriptionSource.parakeetDescription")}
            onClick={() => onViewKindChange("parakeet")}
          />
          <SourceTile
            selected={viewKind === "cloud"}
            active={activeKind === "cloud"}
            icon={<Cloud className="h-5 w-5" />}
            label={t("transcriptionSource.cloudLabel")}
            description={t("transcriptionSource.cloudDescription")}
            onClick={() => onViewKindChange("cloud")}
          />
        </div>
      </CardContent>
    </Card>
  );
}

function SourceTile({
  selected,
  active,
  icon,
  label,
  description,
  onClick,
}: {
  selected: boolean;
  active: boolean;
  icon: React.ReactNode;
  label: string;
  description: string;
  onClick: () => void;
}) {
  const { t } = useTranslation();
  return (
    <button
      onClick={onClick}
      className={cn(
        "relative flex items-start gap-3 rounded-lg border p-4 text-left transition-colors",
        selected ? "border-primary bg-primary/5" : "hover:bg-accent/30",
      )}
    >
      <div
        className={cn(
          "flex h-9 w-9 items-center justify-center rounded-md",
          selected ? "bg-primary text-primary-foreground" : "bg-muted text-foreground",
        )}
      >
        {icon}
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{label}</p>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      {active && (
        <span className="absolute right-2 top-2 inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-medium text-emerald-600 dark:text-emerald-400">
          <Check className="h-3 w-3" />
          {t("transcriptionSource.active")}
        </span>
      )}
    </button>
  );
}
