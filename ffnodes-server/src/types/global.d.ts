// Global type declarations

// Iconify icon web component
declare namespace JSX {
  interface IntrinsicElements {
    'iconify-icon': {
      icon: string;
      class?: string;
      width?: string;
      height?: string;
    };
  }
}
