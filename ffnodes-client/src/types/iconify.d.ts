declare namespace JSX {
  interface IntrinsicElements {
    'iconify-icon': React.DetailedHTMLProps<
      React.HTMLAttributes<HTMLElement> & {
        icon?: string;
        class?: string;
        width?: string | number;
        height?: string | number;
      },
      HTMLElement
    >;
  }
}
