import React, {useEffect, useRef, useState, memo} from "react";
import {TMDBService} from "../services/tmdb";
import {TMDBPosterItem} from "../types/tmdb";
import {motion} from "framer-motion";

interface MoviePosterScrollProps
{
    side: "left" | "right";
    blur?: boolean;
    speed?: number;
    columnIndex?: number;
}

export const MoviePosterScroll: React.FC<MoviePosterScrollProps> = memo(({
                                                                        side,
                                                                        blur = false,
                                                                        speed = 30,
                                                                        columnIndex = 0
                                                                    }) =>
{
    const [posters, setPosters] = useState<TMDBPosterItem[]>([]);
    const [loading, setLoading] = useState(true);
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() =>
    {
        const fetchPosters = async () =>
        {
            try
            {
                const fetchedPosters = await TMDBService.getMixedPopularPosters(5, 3);
                // Duplicate the posters array to create seamless loop
                setPosters([...fetchedPosters, ...fetchedPosters]);
                setLoading(false);
            } catch (error)
            {
                console.error("Error loading posters:", error);
                setLoading(false);
            }
        };

        fetchPosters();
    }, []);

    // Calculate animation duration based on speed
    // Vary speed slightly for each column to create organic movement
    const speedVariation = 1 + (columnIndex * 0.15);
    const animationDuration = posters.length * speed * speedVariation;

    return (
        <div className="relative h-full w-full overflow-hidden">
            {/* Blur overlay */}
            {blur && (
                <div className="absolute inset-0 backdrop-blur-sm bg-black/40 z-10 pointer-events-none"/>
            )}

            {/* Scrolling container */}
            <div
                ref={scrollRef}
                className="relative h-full w-full overflow-hidden"
                style={{perspective: "1000px"}}
            >
                {loading ? (
                    <div className="flex items-center justify-center h-full">
                        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-primary"></div>
                    </div>
                ) : (
                    <motion.div
                        className="flex flex-col gap-4 py-4"
                        animate={{
                            y: [0, -posters.length * 150 / 2]
                        }}
                        transition={{
                            duration: animationDuration,
                            repeat: Infinity,
                            ease: "linear"
                        }}
                    >
                        {posters.map((poster, index) => (
                            <PosterCard
                                key={`${poster.id}-${index}`}
                                poster={poster}
                                side={side}
                            />
                        ))}
                    </motion.div>
                )}
            </div>

            {/* Fade gradients at top and bottom */}
            <div className="absolute top-0 left-0 right-0 h-32 bg-gradient-to-b from-black to-transparent z-20 pointer-events-none"/>
            <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-black to-transparent z-20 pointer-events-none"/>
        </div>
    );
});

MoviePosterScroll.displayName = 'MoviePosterScroll';

interface PosterCardProps
{
    poster: TMDBPosterItem;
    side: "left" | "right";
}

const PosterCard: React.FC<PosterCardProps> = memo(({poster, side}) =>
{
    const [imageLoaded, setImageLoaded] = useState(false);
    const [imageError, setImageError] = useState(false);

    return (
        <motion.div
            className="flex-shrink-0 w-full px-4"
            initial={{opacity: 0, x: side === "left" ? -20 : 20}}
            animate={{opacity: 1, x: 0}}
            transition={{duration: 0.3}}
        >
            <div className="relative aspect-[2/3] rounded-md-lg overflow-hidden shadow-md-3 bg-content2">
                {!imageError ? (
                    <>
                        {!imageLoaded && (
                            <div className="absolute inset-0 flex items-center justify-center bg-content2">
                                <div className="animate-pulse w-full h-full bg-content3"/>
                            </div>
                        )}
                        <img
                            src={poster.posterUrl}
                            alt={poster.title}
                            className={`w-full h-full object-cover transition-opacity duration-300 ${
                                imageLoaded ? "opacity-100" : "opacity-0"
                            }`}
                            onLoad={() => setImageLoaded(true)}
                            onError={() => setImageError(true)}
                            loading="lazy"
                        />
                    </>
                ) : (
                    <div className="absolute inset-0 flex items-center justify-center bg-content2">
                        <iconify-icon
                            icon="mdi:image-broken-variant"
                            class="text-4xl text-foreground/30"
                        />
                    </div>
                )}

                {/* Hover overlay with title */}
                <motion.div
                    className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent opacity-0 hover:opacity-100 transition-opacity duration-300 flex items-end p-4"
                    whileHover={{opacity: 1}}
                >
                    <div className="text-white">
                        <p className="text-sm font-medium line-clamp-2">{poster.title}</p>
                        <p className="text-xs text-white/70 mt-1">
                            {poster.type === "movie" ? "Movie" : "TV Show"}
                        </p>
                    </div>
                </motion.div>
            </div>
        </motion.div>
    );
});

PosterCard.displayName = 'PosterCard';

interface MoviePosterBackgroundProps
{
    children: React.ReactNode;
    className?: string;
}

export const MoviePosterBackground: React.FC<MoviePosterBackgroundProps> = memo(({
                                                                                children,
                                                                                className = ""
                                                                            }) =>
{
    const [columnCount, setColumnCount] = React.useState(0);

    React.useEffect(() =>
    {
        const calculateColumns = () =>
        {
            const columnWidth = 120;
            const gapWidth = 64; // 4rem (64px) gap between columns
            const totalColumnWidth = columnWidth + gapWidth;
            const availableWidth = window.innerWidth;

            // Calculate how many columns can fit
            const columns = Math.floor(availableWidth / totalColumnWidth);
            setColumnCount(Math.max(columns, 2)); // Minimum 2 columns
        };

        calculateColumns();
        window.addEventListener("resize", calculateColumns);
        return () => window.removeEventListener("resize", calculateColumns);
    }, []);

    return (
        <div className={`relative w-full h-full overflow-hidden bg-black ${className}`}>
            {/* Dynamically generated columns of posters */}
            <div className="absolute inset-0 flex justify-center items-center gap-[4rem] z-0">
                {Array.from({length: columnCount}).map((_, index) => (
                    <div
                        key={index}
                        className="h-full w-64 flex-shrink-0"
                    >
                        <MoviePosterScroll
                            side={index % 2 === 0 ? "left" : "right"}
                            speed={25 + (index * 3)}
                            columnIndex={index}
                        />
                    </div>
                ))}
            </div>

            {/* overlay over entire background */}
            <div className="absolute inset-0 bg-black/50 z-[5] pointer-events-none"/>

            {/* Content overlay */}
            <div className="relative z-10 w-full h-full">
                {children}
            </div>
        </div>
    );
});

MoviePosterBackground.displayName = 'MoviePosterBackground';
