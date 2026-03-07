const STEPS = ["环境检测", "安装执行", "安装验证"] as const;

interface StepperProps {
  currentStep: number;
}

export function Stepper({ currentStep }: StepperProps) {
  return (
    <ol className="grid grid-cols-3 gap-2 rounded-xl bg-slate-100 p-3">
      {STEPS.map((label, index) => {
        const done = index < currentStep;
        const active = index === currentStep;
        return (
          <li
            key={label}
            className={`rounded-lg border px-3 py-2 text-sm ${
              active
                ? "border-blue-500 bg-blue-50 text-blue-700"
                : done
                  ? "border-emerald-500 bg-emerald-50 text-emerald-700"
                  : "border-slate-200 bg-white text-slate-500"
            }`}
          >
            <span className="mr-2 inline-flex h-5 w-5 items-center justify-center rounded-full border border-current text-xs">
              {index + 1}
            </span>
            {label}
          </li>
        );
      })}
    </ol>
  );
}
