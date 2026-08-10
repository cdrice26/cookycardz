import { NotificationKind } from '@/context/NotificationContext';
import RequestFn from '@/types/RequestFn';
import { ScheduleDisplay } from '@/types/Schedule';
import { getDaysInMonth, getDatesBetween } from '@/utils/dateUtils';
import { useEffect, useMemo, useState } from 'react';
import useSWR from 'swr';

const firstDay = new Date(new Date().getFullYear(), new Date().getMonth(), 1);
const lastDay = new Date(
  new Date().getFullYear(),
  new Date().getMonth() + 1,
  0
);

export interface RecipeDate {
  date: Date | null;
  recipes: ScheduleDisplay[];
}

export interface ScheduledRecipes {
  isLoading: boolean;
  error: Error | null;
  recipesOnDates: RecipeDate[];
  setYearInput: React.Dispatch<React.SetStateAction<number>>;
  setMonth: React.Dispatch<React.SetStateAction<number>>;
  year: number;
  yearInput: number;
  month: number;
}

/**
 * Hook to load scheduled recipes within a date range.
 *
 * @param request - The request function to use for fetching the scheduled recipes.
 * @param addNotification - Function to add a notification.
 * @param initialStartDate - Initial start date to query scheduled recipes. Defaults to the first day of the current month.
 * @param initialEndDate - Initial end date to query scheduled recipes. Defaults to the last day of the current month.
 * @returns An object containing:
 *  - `recipesOnDates`: Record<string, ScheduleDisplay[]> — the list of scheduled recipes for the current date range
 *  - `error`: Error | null — error object if any error occurred during fetching
 *  - `isLoading`: boolean — whether the data is currently being fetched
 *  - `setYearInput`: (year: number) => void — setter for the year input
 *  - `setMonth`: (month: number) => void — setter for the month
 *  - `year`: number — the current year
 *  - `yearInput`: number — the input year
 *  - `month`: number — the current month
 */
const useScheduledRecipes = (
  request: RequestFn,
  addNotification: (message: string, type: NotificationKind) => void,
  initialStartDate: Date = firstDay,
  initialEndDate: Date = lastDay
) => {
  const [startDate, setStartDate] = useState(initialStartDate);
  const [endDate, setEndDate] = useState(initialEndDate);

  const fetchScheduledRecipes = async () => {
    const resp = await request(
      `/api/recipes/scheduled?startDate=${startDate.toISOString()}&endDate=${endDate.toISOString()}`,
      'GET'
    );
    if (!resp.ok) {
      addNotification('Failed to fetch scheduled recipes.', 'error');
    }
    const json = await resp.json();
    return json?.data?.map((schedule: ScheduleDisplay) => {
      const d = new Date(schedule.scheduledDate);
      // Create a new Date with only year, month, day (time will be 00:00:00)
      const dateOnly = new Date(
        d.getUTCFullYear(),
        d.getUTCMonth(),
        d.getUTCDate()
      );
      return {
        ...schedule,
        scheduledDate: dateOnly
      };
    });
  };

  const {
    data: scheduledRecipes,
    isLoading,
    error,
    mutate
  } = useSWR(
    `/api/recipes/scheduled?startDate=${startDate.toISOString()}&endDate=${endDate.toISOString()}`,
    fetchScheduledRecipes
  );

  const [year, setYear] = useState(new Date().getFullYear());
  const [yearInput, setYearInput] = useState(new Date().getFullYear());
  const [month, setMonth] = useState(new Date().getMonth());

  useEffect(() => {
    setStartDate(new Date(year, month, 1));
    setEndDate(new Date(year, month, getDaysInMonth(year, month)));
  }, [year, month]);

  useEffect(() => {
    if (yearInput < 10000 && yearInput >= 1900 && !isNaN(yearInput)) {
      setYear(yearInput);
    }
  }, [yearInput]);

  const recipesOnDates = useMemo(() => {
    const daysBefore = startDate.getDay();
    const daysAfter = 6 - endDate.getDay();
    const daysInMonth = getDatesBetween(startDate, endDate);

    return [
      ...new Array(daysBefore)
        .fill(null)
        .map(() => ({ date: null, recipes: [] })),
      ...daysInMonth.map((date) => ({
        date,
        recipes: (scheduledRecipes ?? []).filter((recipe: ScheduleDisplay) => {
          const s = recipe.scheduledDate;
          return (
            s.getFullYear() === date.getFullYear() &&
            s.getMonth() === date.getMonth() &&
            s.getDate() === date.getDate()
          );
        })
      })),
      ...new Array(daysAfter)
        .fill(null)
        .map(() => ({ date: null, recipes: [] }))
    ];
  }, [scheduledRecipes, startDate, endDate]);

  return {
    isLoading,
    error,
    recipesOnDates,
    setYearInput,
    setMonth,
    year,
    yearInput,
    month,
    mutate
  };
};

export default useScheduledRecipes;
