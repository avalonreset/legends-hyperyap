import { useState } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { RefreshCcw } from 'lucide-react';
import { useTranslation } from '@/i18n';

type UpdateCheckerProps = {
    className?: string;
};

const RELEASES_URL = 'https://github.com/avalonreset/hyperyap/releases';

export const UpdateChecker = ({ className = '' }: UpdateCheckerProps) => {
    const [clicked, setClicked] = useState(false);
    const { t } = useTranslation();

    const onClick = async () => {
        setClicked(true);
        await openUrl(RELEASES_URL);
        setTimeout(() => setClicked(false), 3000);
    };

    return (
        <button
            onClick={onClick}
            disabled={clicked}
            className={`text-xs transition-colors flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer disabled:opacity-50 text-muted-foreground hover:text-foreground hover:bg-accent ${className}`}
        >
            <RefreshCcw className="w-4 h-4" />
            <span>{clicked ? t('Opening releases...') : t('Check for updates')}</span>
        </button>
    );
};

export default UpdateChecker;
