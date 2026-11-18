import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { Scene3DProps, SceneObject } from '../types';

interface RenderEngine {
    domElement: HTMLCanvasElement;
    setPixelRatio(value: number): void;
    setSize(width: number, height: number, updateStyle?: boolean): void;
    render(scene: THREE.Scene, camera: THREE.Camera): void;
    dispose(): void;
}

const DEFAULT_CAMERA: [number, number, number] = [4.5, 3, 6];
const ORIGIN: [number, number, number] = [0, 0, 0];
const ONE: [number, number, number] = [1, 1, 1];
const HEX = /^#[0-9a-fA-F]{6}$/;
const DEGREE = Math.PI / 180;

function geometryFor(shape: SceneObject['shape']): THREE.BufferGeometry {
    switch (shape) {
        case 'sphere':
            return new THREE.SphereGeometry(0.75, 40, 24);
        case 'cylinder':
            return new THREE.CylinderGeometry(0.65, 0.65, 1.4, 36);
        case 'cone':
            return new THREE.ConeGeometry(0.7, 1.4, 36);
        case 'torus':
            return new THREE.TorusGeometry(0.72, 0.22, 24, 64);
        case 'plane':
            return new THREE.PlaneGeometry(2, 2, 12, 12);
        default:
            return new THREE.BoxGeometry(1.25, 1.25, 1.25, 4, 4, 4);
    }
}

function cssColour(element: HTMLElement, name: string, fallback: string): string {
    const value = getComputedStyle(element).getPropertyValue(name).trim();
    return HEX.test(value) ? value : fallback;
}

async function createRenderer(): Promise<RenderEngine> {
    try {
        if (!(typeof navigator !== 'undefined' && 'gpu' in navigator)) throw new Error('WebGPU is unavailable');
        const module = await import('three/webgpu');
        const renderer = new module.WebGPURenderer({ antialias: true, alpha: true });
        await renderer.init();
        return renderer as unknown as RenderEngine;
    } catch {
        return new THREE.WebGLRenderer({ antialias: true, alpha: true });
    }
}

function dispose(object: THREE.Object3D) {
    object.traverse((child) => {
        if (!(child instanceof THREE.Mesh)) return;
        child.geometry.dispose();
        const materials = Array.isArray(child.material) ? child.material : [child.material];
        for (const material of materials) material.dispose();
    });
}

/**
 * A scene of primitive shapes drawn with `three` (WebGPU where the device has it, else
 * WebGL). It loads no model or texture and answers no click: a scene only shows. Loaded only
 * when used.
 */
export default function SceneBody({ scene: spec, onUnavailable }: { scene: Scene3DProps; onUnavailable: () => void }) {
    const mountRef = useRef<HTMLDivElement>(null);
    const [failed, setFailed] = useState(false);
    const height = Math.min(960, Math.max(240, spec.height ?? 440));

    useEffect(() => {
        const mount = mountRef.current;
        if (!mount || spec.objects.length === 0) return;
        let disposed = false;
        let renderer: RenderEngine | null = null;
        let controls: OrbitControls | null = null;
        let frame = 0;
        let visible = true;
        const scene = new THREE.Scene();
        const world = new THREE.Group();
        scene.add(world);
        if (spec.background && HEX.test(spec.background)) scene.background = new THREE.Color(spec.background);

        const camera = new THREE.PerspectiveCamera(Math.min(90, Math.max(20, spec.camera?.fov ?? 42)), 1, 0.1, 2000);
        camera.position.set(...(spec.camera?.position ?? DEFAULT_CAMERA));
        camera.lookAt(...(spec.camera?.target ?? ORIGIN));
        scene.add(new THREE.HemisphereLight('#ffffff', '#94a3b8', 2.1));
        const key = new THREE.DirectionalLight('#ffffff', 3.2);
        key.position.set(4, 7, 6);
        scene.add(key);

        const accent = cssColour(mount, '--chart-1', '#2563eb');
        for (const object of spec.objects.slice(0, 64)) {
            const mesh = new THREE.Mesh(
                geometryFor(object.shape),
                new THREE.MeshStandardMaterial({
                    color: object.color && HEX.test(object.color) ? object.color : accent,
                    metalness: 0.25,
                    roughness: 0.45,
                }),
            );
            mesh.position.set(...(object.position ?? ORIGIN));
            const [rx, ry, rz] = object.rotation ?? ORIGIN;
            mesh.rotation.set(rx * DEGREE, ry * DEGREE, rz * DEGREE);
            mesh.scale.set(...(object.scale ?? ONE));
            world.add(mesh);
        }

        const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        let last = performance.now();
        const animate = () => {
            if (disposed) return;
            const now = performance.now();
            const delta = (now - last) / 1000;
            last = now;
            if (visible && renderer) {
                if (spec.autoRotate && !reducedMotion) world.rotation.y += delta * 0.16;
                controls?.update();
                renderer.render(scene, camera);
            }
            frame = requestAnimationFrame(animate);
        };
        const resize = new ResizeObserver(([entry]) => {
            if (!renderer || !entry) return;
            const width = Math.max(1, entry.contentRect.width);
            renderer.setSize(width, height, false);
            camera.aspect = width / height;
            camera.updateProjectionMatrix();
        });
        const intersection = new IntersectionObserver(([entry]) => {
            visible = entry?.isIntersecting ?? true;
        });

        void createRenderer()
            .then((created) => {
                if (disposed) {
                    created.dispose();
                    return;
                }
                renderer = created;
                renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
                renderer.domElement.setAttribute('aria-hidden', 'true');
                mount.appendChild(renderer.domElement);
                if (spec.controls === 'orbit') {
                    controls = new OrbitControls(camera, renderer.domElement);
                    controls.enableDamping = true;
                    controls.target.set(...(spec.camera?.target ?? ORIGIN));
                }
                resize.observe(mount);
                intersection.observe(mount);
                frame = requestAnimationFrame(animate);
            })
            .catch(() => {
                setFailed(true);
                onUnavailable();
            });

        return () => {
            disposed = true;
            cancelAnimationFrame(frame);
            resize.disconnect();
            intersection.disconnect();
            controls?.dispose();
            if (renderer) {
                renderer.domElement.remove();
                renderer.dispose();
            }
            dispose(world);
        };
    }, [spec, height, onUnavailable]);

    if (failed) return null;
    return <div ref={mountRef} className="adk-ui-scene__viewport" style={{ height }} />;
}
