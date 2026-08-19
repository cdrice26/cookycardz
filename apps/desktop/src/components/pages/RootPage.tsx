import { Outlet } from 'react-router';
import Sidebar from '../navigation/Sidebar';
import { platform } from '@tauri-apps/plugin-os';
import { useEffect, useRef, useState } from 'react';
import Toolbar from '../navigation/Toolbar';
import { NotificationWrapper, useNotification } from 'cookycardz-shared';
import { useTauriListener } from '../../hooks/useTauriListener';

export default function RootPage() {
  const sidebarRef = useRef<HTMLDivElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);
  const [dragWidth, setDragWidth] = useState(0);

  const { addNotification } = useNotification();

  useEffect(() => {
    if (platform() !== 'macos') return;

    const sidebarNode = sidebarRef.current;
    const contentNode = contentRef.current;
    if (!sidebarNode || !contentNode) return;

    const ro = new ResizeObserver(() => {
      const w = contentNode.getBoundingClientRect().width;
      setDragWidth(window.innerWidth - Math.round(w));
    });

    ro.observe(sidebarNode);

    // Initialize once so your drag region width is correct on mount
    const initialW = contentNode.getBoundingClientRect().width;
    setDragWidth(window.innerWidth - Math.round(initialW));

    return () => {
      ro.unobserve(sidebarNode);
      ro.disconnect();
    };
  }, []);

  useTauriListener('new_recipe_cloud_error', (event) => {
    addNotification(event.payload, 'error');
  });

  useTauriListener('delete_recipe_cloud_error', (event) => {
    addNotification(event.payload, 'error');
  });

  useTauriListener('update_recipe_cloud_error', (event) => {
    addNotification(event.payload, 'error');
  });

  useTauriListener('schedule_update_cloud_error', (event) => {
    addNotification(event.payload, 'error');
  });

  useTauriListener('sync_error', (event) => {
    addNotification(event.payload, 'error');
  });

  useTauriListener('sync_success', (event) => {
    addNotification(event.payload, 'success');
  });

  return (
    <>
      <NotificationWrapper />
      <div
        className="h-10 fixed top-0 left-0 z-50 inset-0 outline-red-900"
        style={{ width: dragWidth + 'px' }}
        data-tauri-drag-region
      ></div>

      <div
        className={`relative w-full h-full flex ${platform() === 'macos' ? 'bg-transparent' : 'bg-white dark:bg-[#202020]'} overflow-hidden`}
      >
        <Sidebar ref={sidebarRef} />
        <div
          ref={contentRef}
          className={`flex-1 flex flex-col z-10 pt-2 bg-white dark:bg-[#202020]`}
        >
          <Toolbar />
          <div className="flex-1 min-h-0 px-2 overflow-y-auto relative">
            <Outlet />
          </div>
        </div>
      </div>
    </>
  );
}
