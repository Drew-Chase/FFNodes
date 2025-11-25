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
                'md-tertiary': {
                    50: '#e0f2f1',
                    100: '#b2dfdb',
                    200: '#80cbc4',
                    300: '#4db6ac',
                    400: '#26a69a',
                    500: '#009688',
                    600: '#00897b',
                    700: '#00796b',
                    800: '#00695c',
                    900: '#004d40',
                },
                'md-surface': {
                    light: '#fafafa',
                    'light-1': '#f5f5f5',
                    'light-2': '#eeeeee',
                    'light-3': '#e0e0e0',
                    dark: '#121212',
                    'dark-1': '#1e1e1e',
                    'dark-2': '#232323',
                    'dark-3': '#252525',
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
                'label-lg': ['14px', { lineHeight: '20px', fontWeight: '500' }],
                'label-md': ['12px', { lineHeight: '16px', fontWeight: '500' }],
                'label-sm': ['11px', { lineHeight: '16px', fontWeight: '500' }],
            },
            // Animation durations (Material Design motion)
            transitionDuration: {
                'md-short-1': '50ms',
                'md-short-2': '100ms',
                'md-short-3': '150ms',
                'md-short-4': '200ms',
                'md-medium-1': '250ms',
                'md-medium-2': '300ms',
                'md-medium-3': '350ms',
                'md-medium-4': '400ms',
                'md-long-1': '450ms',
                'md-long-2': '500ms',
                'md-long-3': '550ms',
                'md-long-4': '600ms',
            },
            // Bento Grid utilities
            gridTemplateColumns: {
                'bento': 'repeat(auto-fit, minmax(250px, 1fr))',
                'bento-sm': 'repeat(auto-fit, minmax(200px, 1fr))',
                'bento-lg': 'repeat(auto-fit, minmax(300px, 1fr))',
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
                    secondary: {
                        DEFAULT: "#e91e63",
                        foreground: "#ffffff",
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
                    success: {
                        DEFAULT: "#009688",
                        foreground: "#ffffff",
                    },
                    background: "#fafafa",
                    foreground: "#1a1a1a",
                    content1: "#ffffff",
                    content2: "#f5f5f5",
                    content3: "#eeeeee",
                    content4: "#e0e0e0",
                }
            },
            dark: {
                colors: {
                    primary: {
                        DEFAULT: "#7986cb",
                        foreground: "#000000",
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
                    secondary: {
                        DEFAULT: "#f48fb1",
                        foreground: "#000000",
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
                    success: {
                        DEFAULT: "#4db6ac",
                        foreground: "#000000",
                    },
                    background: "#121212",
                    foreground: "#e0e0e0",
                    content1: "#1e1e1e",
                    content2: "#232323",
                    content3: "#252525",
                    content4: "#2c2c2c",
                }
            },
        }
    })]
}