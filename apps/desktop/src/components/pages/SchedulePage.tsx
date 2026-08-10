import {
  Schedule,
  useIsDark,
  useNotification,
  useScheduleActions,
  useScheduledRecipes
} from 'cookycardz-shared';
import { useNavigate } from 'react-router';
import { request } from '../../utils/fetchUtils';
import Select from 'react-select';
import { useTauriListener } from '../../hooks/useTauriListener';

export default function SchedulePage() {
  const navigate = useNavigate();
  const { addNotification } = useNotification();
  const data = useScheduledRecipes(request, addNotification);
  const isDark = useIsDark();
  const { handleButtonClick } = useScheduleActions(navigate);

  useTauriListener('sync_success', () => data.mutate());
  useTauriListener('logout', () => data.mutate());
  useTauriListener('import_complete', () => data.mutate());

  return (
    <Schedule
      {...data}
      isDark={isDark}
      handleButtonClick={handleButtonClick}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      SelectComponent={Select as (props: any) => React.ReactNode}
    />
  );
}
