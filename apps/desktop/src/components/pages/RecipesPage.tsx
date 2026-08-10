import {
  Dashboard,
  useInfiniteScroll,
  usePaginatedRecipes
} from 'cookycardz-shared';
import { useMemo, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { request } from '../../utils/fetchUtils';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useTauriListener } from '../../hooks/useTauriListener';

export default function RecipesPage() {
  const navigate = useNavigate();

  const [searchParams] = useSearchParams();
  const query = useMemo(() => searchParams.get('q') ?? '', [searchParams]);
  const tags = useMemo(
    () => searchParams.get('tags')?.split(',').filter(Boolean) ?? [],
    [searchParams]
  );

  const { recipes, page, setPage, hasMore, loading, error, mutate } =
    usePaginatedRecipes(request, query, tags);

  useTauriListener('sync_success', () => mutate());
  useTauriListener('logout', () => mutate());
  useTauriListener('import_complete', () => mutate());

  const updatedRecipes = useMemo(() => {
    const newRecipes = recipes.map((recipe) => ({
      ...recipe,
      imgUrl: recipe?.imgUrl !== null ? convertFileSrc(recipe?.imgUrl) : null,
      tags: (recipe?.tags as unknown as { name: string }[])?.map(
        (tag) => tag?.name
      )
    }));
    return newRecipes;
  }, [recipes]);

  const loaderRef = useRef<HTMLDivElement | null>(null);
  useInfiniteScroll(loaderRef, hasMore, loading, setPage, recipes?.length);

  return (
    <Dashboard
      recipes={updatedRecipes}
      page={page}
      hasMore={hasMore}
      loading={loading}
      loaderRef={loaderRef}
      error={error}
      redirect={navigate}
      handleImageError={() => {}}
      handleImageLoad={() => {}}
    />
  );
}
