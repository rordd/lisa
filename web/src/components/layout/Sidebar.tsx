import { NavLink } from 'react-router-dom';
import {
  LayoutDashboard,
  MessageSquare,
  Wrench,
  Clock,
  Puzzle,
  Brain,
  Wand2,
  Settings,
  DollarSign,
  Activity,
  Stethoscope,
} from 'lucide-react';
import { t } from '@/lib/i18n';
import { useAuth } from '@/hooks/useAuth';

const navItems = [
  { to: '/', icon: LayoutDashboard, labelKey: 'nav.dashboard' },
  { to: '/agent', icon: MessageSquare, labelKey: 'nav.agent' },
  { to: '/tools', icon: Wrench, labelKey: 'nav.tools' },
  { to: '/cron', icon: Clock, labelKey: 'nav.cron' },
  { to: '/integrations', icon: Puzzle, labelKey: 'nav.integrations' },
  { to: '/memory', icon: Brain, labelKey: 'nav.memory' },
  { to: '/setup', icon: Wand2, labelKey: 'nav.setup', onboardOnly: true },
  { to: '/config', icon: Settings, labelKey: 'nav.config' },
  { to: '/cost', icon: DollarSign, labelKey: 'nav.cost' },
  { to: '/logs', icon: Activity, labelKey: 'nav.logs' },
  { to: '/doctor', icon: Stethoscope, labelKey: 'nav.doctor' },
];

export default function Sidebar() {
  const { serverMode } = useAuth();
  const isOnboard = serverMode === 'onboard';

  const visibleItems = isOnboard
    ? navItems.filter((i) => i.onboardOnly)
    : navItems.filter((i) => !i.onboardOnly);

  return (
    <aside className="fixed top-0 left-0 h-screen w-60 bg-gray-900 flex flex-col border-r border-gray-800">
      {/* Logo / Title */}
      <div className="flex items-center gap-2 px-5 py-5 border-b border-gray-800">
        <div className="h-8 w-8 rounded-lg bg-blue-600 flex items-center justify-center text-white font-bold text-sm">
          ZC
        </div>
        <span className="text-lg font-semibold text-white tracking-wide">
          ZeroClaw
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 overflow-y-auto py-4 px-3 space-y-1">
        {visibleItems.map(({ to, icon: Icon, labelKey }) => (
          <NavLink
            key={to}
            to={to}
            end={to === '/'}
            className={({ isActive }) =>
              [
                'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors',
                isActive
                  ? 'bg-blue-600 text-white'
                  : 'text-gray-300 hover:bg-gray-800 hover:text-white',
              ].join(' ')
            }
          >
            <Icon className="h-5 w-5 flex-shrink-0" />
            <span>{t(labelKey)}</span>
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
