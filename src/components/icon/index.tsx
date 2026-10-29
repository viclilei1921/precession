type IconProps = {
  className?: string;
};

/** 品牌图标 */
export function Icon({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 390 390" aria-hidden="true">
      <polygon points="195,0 292.5,97.5 195,195 97.5,97.5" fill="#FFE270" />
      <polygon points="390,195 292.5,292.5 195,195 292.5,97.5" fill="#D8CDE3" />
      <polygon points="195,390 97.5,292.5 195,195 292.5,292.5" fill="#89A9C2" />
      <polygon points="0,195 97.5,97.5 195,195 97.5,292.5" fill="#9CC8B5" />
      <rect x="168" y="168" width="55" height="55" fill="#F8F9FA" />
    </svg>
  );
}
