import { useEffect, useRef } from "react";

interface Star {
  x: number;
  y: number;
  speed: number; // 1, 2, or 3 (frames between each move)
  brightness: number; // 0.3, 0.6, or 1.0
  size: number; // 1, 2, or 3 px radius
  tick: number;
}

const STAR_COUNT = 60;

function createStar(width: number, height: number): Star {
  const tier = Math.floor(Math.random() * 3);
  return {
    x: Math.random() * width,
    y: Math.random() * height,
    speed: [3, 2, 1][tier],
    brightness: [0.3, 0.6, 1.0][tier],
    size: [1, 1.5, 2.5][tier],
    tick: 0,
  };
}

export default function Starfield() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const starsRef = useRef<Star[]>([]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    resize();
    window.addEventListener("resize", resize);

    // Initialize stars
    starsRef.current = Array.from({ length: STAR_COUNT }, () =>
      createStar(canvas.width, canvas.height),
    );

    let animId: number;

    const draw = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      for (const star of starsRef.current) {
        star.tick++;
        if (star.tick >= star.speed) {
          star.tick = 0;
          star.y += 1;
          // Wrap around
          if (star.y > canvas.height) {
            star.y = 0;
            star.x = Math.random() * canvas.width;
          }
        }

        ctx.beginPath();
        ctx.arc(star.x, star.y, star.size, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(255, 255, 255, ${star.brightness})`;
        ctx.fill();
      }

      animId = requestAnimationFrame(draw);
    };

    animId = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="pointer-events-none fixed inset-0 z-0"
      aria-hidden="true"
    />
  );
}
