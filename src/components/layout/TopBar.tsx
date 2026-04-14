interface TopBarProps {
  title: string;
  subtitle?: string;
  actions?: React.ReactNode;
}

export function TopBar({ title, subtitle, actions }: TopBarProps) {
  return (
    <header className="flex h-16 items-center justify-between border-b border-gray-200 bg-white px-6">
      <div>
        <h1 className="text-lg font-semibold text-gray-900">{title}</h1>
        {subtitle && (
          <p className="text-sm text-gray-500">{subtitle}</p>
        )}
      </div>
      {actions && <div className="flex items-center gap-3">{actions}</div>}
    </header>
  );
}
