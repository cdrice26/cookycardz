import { platform } from '@tauri-apps/plugin-os';
import {
  IoIosArrowBack,
  IoIosArrowForward,
  IoIosClose,
  IoIosPerson,
  IoIosSearch,
  IoIosSync
} from 'react-icons/io';
import { IoAddCircleOutline } from 'react-icons/io5';
import { useNavigate, useSearchParams } from 'react-router';
import { useEffect, useState } from 'react';
import { useAuth } from '../../hooks/useAuth';
import { useSync } from '../../hooks/useSync';
import { useLocation } from 'react-router';

export default function Toolbar() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const { username } = useAuth();
  const { sync } = useSync();
  const [searchParams, setSearchParams] = useSearchParams();
  const [query, setQuery] = useState(searchParams.get('q') ?? '');

  // Keep local input state in sync when search params change externally
  useEffect(() => {
    setQuery(searchParams.get('q') ?? '');
  }, [searchParams]);

  // Debounce updating the URL search params when the user types
  useEffect(() => {
    const handler = setTimeout(() => {
      setSearchParams(
        (prev) => {
          const next = new URLSearchParams(prev);
          if (query) next.set('q', query);
          else next.delete('q');
          return next;
        },
        { replace: true }
      );
    }, 300);

    return () => clearTimeout(handler);
  }, [query, setSearchParams]);

  return (
    <div
      data-tauri-drag-region={platform() === 'macos'}
      className={`sticky flex flex-row items-center justify-between z-10 px-2 ${
        platform() !== 'macos' ? 'bg-white dark:bg-[#202020] shadow-md p-1' : ''
      }`}
    >
      <div
        className={`p-1 h-full flex gap-2 justify-center items-center ${
          platform() === 'macos'
            ? 'rounded-full border border-white dark:border-[#303030] bg-[#fefefe] dark:bg-[#151515] drop-shadow-2xl'
            : ''
        }`}
      >
        <button
          className="rounded-full p-1 hover:bg-gray-100 dark:hover:bg-[#505050] flex justify-center items-center"
          onClick={() => {
            navigate(-1);
          }}
        >
          <IoIosArrowBack className="pointer-events-none text-black dark:text-white" />
        </button>
        <button
          className="rounded-full p-1 hover:bg-gray-100 dark:hover:bg-[#505050] flex justify-center items-center"
          onClick={() => navigate(1)}
        >
          <IoIosArrowForward className="pointer-events-none text-black dark:text-white" />
        </button>
      </div>
      <div className="flex h-full flex-row gap-2 items-center justify-center">
        <div
          className={`p-1 h-full flex gap-2 justify-center items-center ${
            platform() === 'macos'
              ? 'rounded-full border border-white dark:border-[#303030] bg-[#fefefe] dark:bg-[#151515] drop-shadow-2xl'
              : ''
          }`}
        >
          <button
            className="rounded-full h-full p-1 hover:bg-gray-100 dark:hover:bg-[#505050] flex justify-center items-center"
            onClick={() => navigate('/create')}
          >
            <IoAddCircleOutline className="pointer-events-none text-black dark:text-white" />
          </button>
          <button
            className="rounded-full h-full p-1 hover:bg-gray-100 dark:hover:bg-[#505050] flex justify-center items-center"
            onClick={() => navigate('/profile')}
          >
            <IoIosPerson className="pointer-events-none text-black dark:text-white" />
          </button>
          {username && (
            <button
              className="rounded-full h-full p-1 hover:bg-gray-100 dark:hover:bg-[#505050] flex justify-center items-center"
              onClick={() => {
                sync();
              }}
            >
              <IoIosSync className="pointer-events-none text-black dark:text-white" />
            </button>
          )}
        </div>
        <div
          className={`p-1 h-full flex gap-2 justify-center items-center text-sm text-black dark:text-white outline-none font-light ${
            platform() === 'macos'
              ? 'rounded-full border border-white dark:border-[#303030] bg-[#fefefe] dark:bg-[#151515] drop-shadow-2xl'
              : 'rounded-lg bg-gray-100 dark:bg-[#505050]'
          } ${pathname === '/' || pathname === '/dashboard' ? '' : 'hidden'}`}
        >
          <IoIosSearch className="ml-1 aspect-square h-full pointer-events-none text-black dark:text-white" />
          <input
            className="py-1 pr-1 outline-none h-full"
            placeholder="Search"
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          <button
            className="h-full mr-1"
            onClick={() => {
              setQuery('');
              setSearchParams(
                (prev) => {
                  const next = new URLSearchParams(prev);
                  next.delete('q');
                  return next;
                },
                { replace: true }
              );
            }}
          >
            <IoIosClose className="h-full pointer-events-none text-black dark:text-white" />
          </button>
        </div>
      </div>
    </div>
  );
}
