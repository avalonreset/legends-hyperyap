import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { toast } from 'react-toastify';
import { Cpu } from 'lucide-react';
import { SettingsUI } from '@/components/settings-ui';
import { Typography } from '@/components/typography';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/select';
import { useTranslation } from '@/i18n';
import type { AsrModelInfo } from '@/features/settings/settings.types';

export const ModelSettings = () => {
    const { t } = useTranslation();
    const [models, setModels] = useState<AsrModelInfo[]>([]);
    const [asrModel, setAsrModel] = useState('');
    const [isLoading, setIsLoading] = useState(true);

    const loadModels = useCallback(async () => {
        try {
            const [availableModels, selectedModel] = await Promise.all([
                invoke<AsrModelInfo[]>('list_asr_models'),
                invoke<string>('get_asr_model'),
            ]);
            setModels(availableModels);
            setAsrModel(selectedModel);
        } catch (error) {
            console.error('Failed to load ASR models:', error);
            toast.error(t('Failed to load speech models'));
        } finally {
            setIsLoading(false);
        }
    }, [t]);

    useEffect(() => {
        void loadModels();
    }, [loadModels]);

    const selectedModel = useMemo(() => models.find((model) => model.id === asrModel), [asrModel, models]);

    const setModel = async (modelId: string) => {
        const previousModel = asrModel;
        setAsrModel(modelId);

        try {
            await invoke('set_asr_model', { modelId });
            toast.info(t('Speech model updated'));
            await loadModels();
        } catch (error) {
            console.error('Failed to save ASR model:', error);
            setAsrModel(previousModel);
            toast.error(t('Failed to save speech model'));
        }
    };

    return (
        <SettingsUI.Item>
            <SettingsUI.Description>
                <Typography.Title className="flex items-center gap-2">
                    <Cpu className="w-4 h-4 text-muted-foreground" />
                    {t('Speech model')}
                </Typography.Title>
                <Typography.Paragraph>
                    {selectedModel?.available === false
                        ? t('Selected model is not installed.')
                        : t('Local speech-to-text engine.')}
                </Typography.Paragraph>
            </SettingsUI.Description>
            <Select value={asrModel} onValueChange={setModel} disabled={isLoading || models.length === 0}>
                <SelectTrigger className="w-[280px]" data-testid="asr-model-select">
                    <SelectValue placeholder={t('Select a model')} />
                </SelectTrigger>
                <SelectContent>
                    {models.map((model) => (
                        <SelectItem key={model.id} value={model.id} disabled={!model.available}>
                            {model.available ? model.name : t('{{name}} (not installed)', { name: model.name })}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </SettingsUI.Item>
    );
};
