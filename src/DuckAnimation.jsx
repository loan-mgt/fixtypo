import { useState, useEffect, useCallback } from "react";
import { listen, emit } from "@tauri-apps/api/event";
import "./DuckAnimation.css";

// Import all 11 frames
import frame01 from "./assets/duck/duck_anim_01.png";
import frame02 from "./assets/duck/duck_anim_02.png";
import frame03 from "./assets/duck/duck_anim_03.png";
import frame04 from "./assets/duck/duck_anim_04.png";
import frame05 from "./assets/duck/duck_anim_05.png";
import frame06 from "./assets/duck/duck_anim_06.png";
import frame07 from "./assets/duck/duck_anim_07.png";
import frame08 from "./assets/duck/duck_anim_08.png";
import frame09 from "./assets/duck/duck_anim_09.png";
import frame10 from "./assets/duck/duck_anim_10.png";
import frame11 from "./assets/duck/duck_anim_11.png";

const frames = [frame01, frame02, frame03, frame04, frame05, frame06, frame07, frame08, frame09, frame10, frame11];

const FRAME_DURATION = 150; // ms per frame

function DuckAnimation() {
    const [phase, setPhase] = useState("intro"); // intro, running, outro, done
    const [frameIndex, setFrameIndex] = useState(0);

    // Listen for events from Rust backend
    useEffect(() => {
        console.log("DuckAnimation mounted, setting up listeners...");
        const setupListener = async () => {
            const unlisten = await listen("animation-phase", (event) => {
                console.log("Received animation-phase event:", event.payload);
                if (event.payload === "start") {
                    setPhase("intro");
                    setFrameIndex(0);
                } else if (event.payload === "finish") {
                    setPhase("outro");
                    setFrameIndex(7); // Frame 8 (index 7)
                }
            });
            return unlisten;
        };

        const unlistenPromise = setupListener();

        // Start intro on mount
        setPhase("intro");
        setFrameIndex(0);

        return () => {
            unlistenPromise.then(unlisten => unlisten());
        };
    }, []);

    // Animation loop
    useEffect(() => {
        if (phase === "done") return;

        const timer = setInterval(() => {
            setFrameIndex((prev) => {
                if (phase === "intro") {
                    // Frames 0-4 (1-5), then switch to running
                    if (prev >= 4) {
                        setPhase("running");
                        return 5; // Start at frame 6 (index 5)
                    }
                    return prev + 1;
                } else if (phase === "running") {
                    // Loop frames 5-6 (6-7)
                    return prev === 5 ? 6 : 5;
                } else if (phase === "outro") {
                    // Frames 7-10 (8-11), then done
                    if (prev >= 10) {
                        setPhase("done");
                        // Notify Rust that animation is complete
                        emit("animation-complete");
                        return 10;
                    }
                    return prev + 1;
                }
                return prev;
            });
        }, FRAME_DURATION);

        return () => clearInterval(timer);
    }, [phase]);

    if (phase === "done") return null;

    return (
        <div className="duck-container">
            <img
                src={frames[frameIndex]}
                alt="Duck animation"
                className="duck-sprite"
                draggable={false}
            />
        </div>
    );
}

export default DuckAnimation;
