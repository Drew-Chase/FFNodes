import {heroui} from "@heroui/react";

/** @type {import('tailwindcss').Config} */
export default {
    content: [
        "./index.html",
        "./src/**/*.{js,ts,jsx,tsx}",
        "./node_modules/@heroui/theme/dist/**/*.{js,ts,jsx,tsx}"
    ],
    theme: {
        extend: {
            // Material Design 3 inspired color system
            colors: {
                'md-primary': {
                    50: '#e8eaf6',
                    100: '#c5cae9',
                    200: '#9fa8da',
                    300: '#7986cb',
                    400: '#5c6bc0',
                    500: '#3f51b5',
                    600: '#3949ab',
                    700: '#303f9f',
                    800: '#283593',
                    900: '#1a237e',
                },
                'md-secondary': {
                    50: '#fce4ec',
                    100: '#f8bbd0',
                    200: '#f48fb1',
                    300: '#f06292',
                    400: '#ec407a',
                    500: '#e91e63',
                    600: '#d81b60',
                    700: '#c2185b',
                    800: '#ad1457',
                    900: '#880e4f',
                },
            },
            // Material Design elevation shadows
            boxShadow: {
                'md-1': '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
                'md-2': '0 2px 4px 0 rgba(0, 0, 0, 0.06)',
                'md-3': '0 4px 8px 0 rgba(0, 0, 0, 0.08)',
                'md-4': '0 6px 12px 0 rgba(0, 0, 0, 0.1)',
                'md-5': '0 8px 16px 0 rgba(0, 0, 0, 0.12)',
                'md-6': '0 12px 24px 0 rgba(0, 0, 0, 0.15)',
            },
            // Material Design border radius
            borderRadius: {
                'md-xs': '4px',
                'md-sm': '8px',
                'md': '12px',
                'md-lg': '16px',
                'md-xl': '20px',
                'md-2xl': '28px',
            },
            // Material Design typography
            fontSize: {
                'display-lg': ['57px', { lineHeight: '64px', fontWeight: '400' }],
                'display-md': ['45px', { lineHeight: '52px', fontWeight: '400' }],
                'display-sm': ['36px', { lineHeight: '44px', fontWeight: '400' }],
                'headline-lg': ['32px', { lineHeight: '40px', fontWeight: '400' }],
                'headline-md': ['28px', { lineHeight: '36px', fontWeight: '400' }],
                'headline-sm': ['24px', { lineHeight: '32px', fontWeight: '400' }],
                'title-lg': ['22px', { lineHeight: '28px', fontWeight: '500' }],
                'title-md': ['16px', { lineHeight: '24px', fontWeight: '500' }],
                'title-sm': ['14px', { lineHeight: '20px', fontWeight: '500' }],
                'body-lg': ['16px', { lineHeight: '24px', fontWeight: '400' }],
                'body-md': ['14px', { lineHeight: '20px', fontWeight: '400' }],
                'body-sm': ['12px', { lineHeight: '16px', fontWeight: '400' }],
            },
            // Animation durations
            transitionDuration: {
                'md-medium-2': '300ms',
            },
            // Bento Grid utilities
            gridTemplateColumns: {
                'bento': 'repeat(auto-fit, minmax(250px, 1fr))',
            },
        },
    },
    darkMode: "class",
    plugins: [heroui({
        themes: {
            light: {
                colors: {
                    primary: {
                        DEFAULT: "#3f51b5",
                        foreground: "#ffffff",
                    },
                    secondary: {
                        DEFAULT: "#e91e63",
                        foreground: "#ffffff",
                    },
                    success: {
                        DEFAULT: "#009688",
                        foreground: "#ffffff",
                    },
                    background: "#fafafa",
                    foreground: "#1a1a1a",
                    content1: "#ffffff",
                    content2: "#f5f5f5",
                }
            },
            dark: {
                colors: {
                    primary: {
                        DEFAULT: "#7986cb",
                        foreground: "#000000",
                    },
                    secondary: {
                        DEFAULT: "#f48fb1",
                        foreground: "#000000",
                    },
                    success: {
                        DEFAULT: "#4db6ac",
                        foreground: "#000000",
                    },
                    background: "#121212",
                    foreground: "#e0e0e0",
                    content1: "#1e1e1e",
                    content2: "#232323",
                }
            },
        }
    })]
}
