import { motion } from 'framer-motion';
import { useJobStore } from '../stores/useJobStore';
import { useEffect, useState } from 'react';

export function VideoBackground() {
  const { currentProgress } = useJobStore();
  const [imageLoaded, setImageLoaded] = useState(false);

  const extractedFrame = currentProgress?.extractedFrame;
  const progress = currentProgress?.percentage || 0;

  useEffect(() => {
    if (extractedFrame) {
      setImageLoaded(false);
      const img = new Image();
      img.onload = () => setImageLoaded(true);
      img.src = extractedFrame;
    }
  }, [extractedFrame]);

  if (!extractedFrame) {
    // Default gradient background when no video is processing
    return (
      <div className="video-background">
        <div className="absolute inset-0 bg-gradient-to-br from-background via-default-100 to-default-200" />
      </div>
    );
  }

  return (
    <div className="video-background wipe-reveal">
      {/* Base Layer: Black & White, Blurred, Darkened */}
      <motion.div
        className="absolute inset-0"
        initial={{ opacity: 0 }}
        animate={{ opacity: imageLoaded ? 1 : 0 }}
        transition={{ duration: 0.5 }}
      >
        <img
          src={extractedFrame}
          alt="Video background"
          className="w-full h-full object-cover"
          style={{
            filter: 'blur(10px) grayscale(1)',
          }}
        />
      </motion.div>

      {/* Wipe Reveal Layer: Color, Blurred, Darkened */}
      <motion.div
        className="wipe-reveal-layer"
        initial={{ clipPath: 'inset(0 100% 0 0)' }}
        animate={{
          clipPath: `inset(0 ${100 - progress}% 0 0)`,
        }}
        transition={{
          duration: 0.5,
          ease: 'easeOut',
        }}
      >
        <img
          src={extractedFrame}
          alt="Video background color"
          className="w-full h-full object-cover"
          style={{
            filter: 'blur(10px)',
          }}
        />
      </motion.div>

      {/* Dark Overlay */}
      <div className="absolute inset-0 bg-black bg-opacity-60 pointer-events-none" />

      {/* Progress Indicator Line */}
      <motion.div
        className="absolute top-0 bottom-0 w-1 z-10 pointer-events-none bg-primary"
        style={{
          left: `${progress}%`,
        }}
        initial={{ opacity: 0 }}
        animate={{ opacity: progress > 0 && progress < 100 ? 1 : 0 }}
      />
    </div>
  );
}
