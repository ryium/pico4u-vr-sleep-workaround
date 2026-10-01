import { useAppContext } from '../context/AppContext'

export function Logs() {
  const { logs } = useAppContext()

  return (
    <div className='flex flex-col gap-3'>
      <div className='bg-gray-900 dark:bg-black text-gray-200 p-4 rounded-lg h-[240px] overflow-y-auto font-mono text-xs leading-relaxed border border-gray-800 dark:border-gray-800'>
        {logs.map((log, i) => (
          <div key={i} className='mb-1 break-all'>
            {log}
          </div>
        ))}
        {logs.length === 0 && <div className='text-gray-500 italic'>No logs available.</div>}
      </div>
    </div>
  )
}
