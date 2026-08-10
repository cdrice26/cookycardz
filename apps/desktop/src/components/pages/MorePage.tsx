import {
  Button,
  ManageTags,
  ResponsiveForm,
  useAvailableTags,
  useNotification,
  useTagsMutator
} from 'cookycardz-shared';
import { request } from '../../utils/fetchUtils';
import { confirm } from '@tauri-apps/plugin-dialog';
import { FormEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LicensesDialog } from '../LicensesDialog';

export default function MorePage() {
  const { availableTags, error, isLoading, refetch } =
    useAvailableTags(request);
  const { addNotification } = useNotification();

  const mutator = useTagsMutator(request, addNotification, refetch);

  const handleBackupAction = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const action = (e.nativeEvent as SubmitEvent).submitter?.getAttribute(
      'value'
    );
    if (action === 'export') {
      await invoke('zip_data');
    } else if (action === 'import') {
      if (
        await confirm(
          'Are you sure you want to import data? ALL your existing CookyCardz data will be replaced!'
        )
      )
        try {
          await invoke('unzip_data');
          addNotification('Import complete!', 'success');
        } catch (e) {
          addNotification(
            'Error importing data. Please check your archive.',
            'error'
          );
        }
    }
  };
  return (
    <div>
      <ResponsiveForm onSubmit={handleBackupAction}>
        <h1 className="text-2xl font-bold mb-4">Import/Export Database</h1>
        Archives include all your recipes, schedules, and tags, as well as their cloud sync state. Loading an archive will replace all your existing data, sign you out of your current cloud account, and restart the app.
        <Button type="submit" name="action" value="export">
          Export All Recipes to Archive
        </Button>
        <Button type="submit" name="action" value="import">
          Import All Recipes from Archive
        </Button>
      </ResponsiveForm>
      <ManageTags
        {...mutator}
        availableTags={availableTags}
        error={error}
        isLoading={isLoading}
      />
      <LicensesDialog />
    </div>
  );
}
