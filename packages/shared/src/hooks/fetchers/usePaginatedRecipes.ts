import { useState, useMemo } from 'react';
import { Recipe } from '@/types/Recipe';
import RequestFn from '@/types/RequestFn';
import useSWRInfinite, { SWRInfiniteResponse, unstable_serialize } from 'swr/infinite';

const PAGE_SIZE = 20;

/**
 * Custom hook to fetch recipes with pagination and optional filters.
 *
 * The hook automatically loads pages of recipes and deduplicates results when
 * appending subsequent pages. It resets state when `query` or `tags` change.
 *
 * @param request - The request function to use for fetching data.
 * @param query - Search query string used to filter recipes.
 * @param tags - Array of tag strings to filter recipes by.
 * @returns An object containing:
 *  - `recipes`: current list of Recipe items
 *  - `page`: current page number
 *  - `setPage`: function to update the page number
 *  - `hasMore`: whether there are more pages to load
 *  - `loading`: whether a fetch is in progress
 *  - `error`: error message if the last fetch failed
 */
export default function usePaginatedRecipes(
  request: RequestFn,
  query: string,
  tags: string[]
) {
  const [hasMore, setHasMore] = useState(true);

  const getKey = (pageIndex: number, previousPageData: Recipe[]) => {
    if (previousPageData && !previousPageData.length) return null;
    return `/api/recipes?page=${pageIndex + 1}&limit=${PAGE_SIZE}&q=${query}${tags.length > 0 ? `&tags=${tags.join(',')}` : ''}`;
  }

  const fetchRecipes = async (url: string) => {
    try {
      const response = await request(url, 'GET');
      if (!response.ok) {
        throw new Error('Failed to fetch recipes');
      }
      const json = await response.json() as {data: Recipe[]};
      return json?.data;
    } catch (err: any) {
      throw new Error(err.message || 'An error occurred while fetching recipes');
    }
  };

  const { data, isLoading, error, mutate, size, setSize }: SWRInfiniteResponse<Recipe[]> = useSWRInfinite(getKey, fetchRecipes);
  const recipes = useMemo(() => data ? data.flat() : [], [data]);

  return {
    recipes,
    mutate,
    page: size,
    setPage: setSize,
    hasMore,
    loading: isLoading,
    error,
  };
}
