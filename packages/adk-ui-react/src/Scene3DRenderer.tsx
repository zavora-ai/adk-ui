import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { Scene3dComponent, Scene3dMaterialRole, UiEvent } from './types';
import { useAdkUiKit } from './kit';
import { sanitizeUrl } from './security';

interface Scene3DRendererProps {
    component: Scene3dComponent;
    onAction?: (event: UiEvent) => void;
}

interface RenderEngine {
    domElement: HTMLCanvasElement;
    setPixelRatio(value: number): void;
    setSize(width: number, height: number, updateStyle?: boolean): void;
    render(scene: THREE.Scene, camera: THREE.Camera): void;
    dispose(): void;
}

const DEFAULT_POSITION: [number, number, number] = [4.5, 3, 6];
const DEFAULT_TARGET: [number, number, number] = [0, 0, 0];
const DEFAULT_SCALE: [number, number, number] = [1, 1, 1];

function applyTransform(
    target: THREE.Object3D,
    position = DEFAULT_TARGET,
    rotation = DEFAULT_TARGET,
    scale = DEFAULT_SCALE,
) {
    target.position.set(...position);
    target.rotation.set(...rotation);
    target.scale.set(...scale);
}

function geometryFor(shape: Extract<Scene3dComponent['objects'][number], { kind: 'primitive' }>['shape']) {
    switch (shape) {
        case 'sphere': return new THREE.SphereGeometry(0.75, 40, 24);
        case 'cylinder': return new THREE.CylinderGeometry(0.65, 0.65, 1.4, 36);
        case 'torus': return new THREE.TorusGeometry(0.72, 0.22, 24, 64);
        case 'plane': return new THREE.PlaneGeometry(2, 2, 12, 12);
        case 'box': return new THREE.BoxGeometry(1.25, 1.25, 1.25, 4, 4, 4);
    }
}

function colorFor(role: Scene3dMaterialRole | undefined, colors: ReturnType<typeof useAdkUiKit>['manifest']['tokens']['colors']) {
    switch (role) {
        case 'secondary': return colors.secondary;
        case 'surface': return colors.surface;
        case 'muted': return colors.muted;
        case 'success': return colors.success;
        case 'warning': return colors.warning;
        case 'info': return colors.info;
        case 'primary':
        default: return colors.primary;
    }
}

function disposeObject(object: THREE.Object3D) {
    object.traverse((child) => {
        if (!(child instanceof THREE.Mesh)) return;
        child.geometry.dispose();
        const materials = Array.isArray(child.material) ? child.material : [child.material];
        for (const material of materials) {
            for (const value of Object.values(material)) {
                if (value instanceof THREE.Texture) value.dispose();
            }
            material.dispose();
        }
    });
}

async function createRenderer(): Promise<RenderEngine> {
    try {
        if (!(typeof navigator !== 'undefined' && 'gpu' in navigator)) {
            throw new Error('WebGPU is unavailable');
        }
        const module = await import('three/webgpu');
        const renderer = new module.WebGPURenderer({ antialias: true, alpha: true });
        await renderer.init();
        return renderer as unknown as RenderEngine;
    } catch {
        return new THREE.WebGLRenderer({ antialias: true, alpha: true });
    }
}

export default function Scene3DRenderer({ component, onAction }: Scene3DRendererProps) {
    const mountRef = useRef<HTMLDivElement>(null);
    const onActionRef = useRef(onAction);
    const { manifest } = useAdkUiKit();
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        onActionRef.current = onAction;
    }, [onAction]);

    const height = Math.min(960, Math.max(240, component.height ?? 440));
    const labelledObjects = component.objects.filter((object) => object.label);

    useEffect(() => {
        const mount = mountRef.current;
        if (!mount || component.objects.length === 0 || component.objects.length > 64) return;
        let disposed = false;
        let renderer: RenderEngine | null = null;
        let controls: OrbitControls | null = null;
        let animationFrame = 0;
        let visible = true;
        const scene = new THREE.Scene();
        const world = new THREE.Group();
        const interactive: THREE.Object3D[] = [];
        scene.add(world);

        const backgroundRole = component.background ?? 'surface';
        if (backgroundRole !== 'transparent') {
            const background = backgroundRole === 'contrast'
                ? manifest.tokens.colors.foreground
                : backgroundRole === 'muted'
                    ? manifest.tokens.colors.muted
                    : manifest.tokens.colors.surface;
            scene.background = new THREE.Color(background);
        }

        const camera = new THREE.PerspectiveCamera(
            Math.min(90, Math.max(20, component.camera?.fov ?? 42)),
            1,
            0.1,
            1000,
        );
        camera.position.set(...(component.camera?.position ?? DEFAULT_POSITION));
        camera.lookAt(...(component.camera?.target ?? DEFAULT_TARGET));

        scene.add(new THREE.HemisphereLight('#ffffff', manifest.tokens.colors.muted, 2.1));
        const keyLight = new THREE.DirectionalLight('#ffffff', 3.2);
        keyLight.position.set(4, 7, 6);
        scene.add(keyLight);
        const rimLight = new THREE.PointLight(manifest.tokens.colors.primary, 18, 24);
        rimLight.position.set(-4, 1, -3);
        scene.add(rimLight);

        for (const object of component.objects.slice(0, 64)) {
            if (object.kind === 'primitive') {
                const mesh = new THREE.Mesh(
                    geometryFor(object.shape),
                    new THREE.MeshStandardMaterial({
                        color: colorFor(object.material, manifest.tokens.colors),
                        metalness: object.material === 'surface' ? 0.15 : 0.35,
                        roughness: object.material === 'surface' ? 0.58 : 0.34,
                    }),
                );
                mesh.name = object.id;
                mesh.userData.actionId = object.action_id;
                applyTransform(mesh, object.position, object.rotation, object.scale);
                world.add(mesh);
                if (object.action_id) interactive.push(mesh);
                continue;
            }

            const asset = manifest.assets.find((candidate) => candidate.id === object.asset_id);
            if (!asset || asset.kind !== 'model_3d' || asset.mimeType !== 'model/gltf-binary') continue;
            const source = sanitizeUrl(asset.uri, 'media');
            if (!source) continue;
            void new GLTFLoader().loadAsync(source).then((gltf) => {
                if (disposed) {
                    disposeObject(gltf.scene);
                    return;
                }
                gltf.scene.name = object.id;
                gltf.scene.userData.actionId = object.action_id;
                applyTransform(gltf.scene, object.position, object.rotation, object.scale);
                gltf.scene.traverse((child) => { child.userData.actionId = object.action_id; });
                world.add(gltf.scene);
                if (object.action_id) interactive.push(gltf.scene);
            }).catch(() => undefined);
        }

        const pointer = new THREE.Vector2();
        const raycaster = new THREE.Raycaster();
        const handlePointer = (event: PointerEvent) => {
            if (!renderer || interactive.length === 0) return;
            const rect = renderer.domElement.getBoundingClientRect();
            pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            raycaster.setFromCamera(pointer, camera);
            const hit = raycaster.intersectObjects(interactive, true)[0]?.object;
            const actionId = hit?.userData.actionId as string | undefined;
            if (actionId) {
                onActionRef.current?.({ action: 'button_click', action_id: actionId, source_component_id: hit.name || component.id });
            }
        };

        const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
        const clock = new THREE.Clock();
        const animate = () => {
            if (disposed) return;
            if (visible && renderer) {
                if (component.auto_rotate && !reducedMotion) world.rotation.y += clock.getDelta() * 0.16;
                else clock.getDelta();
                controls?.update();
                renderer.render(scene, camera);
            }
            animationFrame = requestAnimationFrame(animate);
        };

        const resizeObserver = new ResizeObserver(([entry]) => {
            if (!renderer || !entry) return;
            const width = Math.max(1, entry.contentRect.width);
            renderer.setSize(width, height, false);
            camera.aspect = width / height;
            camera.updateProjectionMatrix();
        });
        const intersectionObserver = new IntersectionObserver(([entry]) => {
            visible = entry?.isIntersecting ?? true;
        });

        void createRenderer().then((createdRenderer) => {
            if (disposed) {
                createdRenderer.dispose();
                return;
            }
            renderer = createdRenderer;
            renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
            renderer.domElement.setAttribute('aria-hidden', 'true');
            renderer.domElement.addEventListener('pointerdown', handlePointer);
            mount.appendChild(renderer.domElement);
            controls = new OrbitControls(camera, renderer.domElement);
            controls.enabled = component.controls ?? true;
            controls.enableDamping = true;
            controls.target.set(...(component.camera?.target ?? DEFAULT_TARGET));
            controls.update();
            resizeObserver.observe(mount);
            intersectionObserver.observe(mount);
            animationFrame = requestAnimationFrame(animate);
        }).catch(() => setError(component.fallback ?? 'This device cannot render the 3D scene.'));

        return () => {
            disposed = true;
            cancelAnimationFrame(animationFrame);
            resizeObserver.disconnect();
            intersectionObserver.disconnect();
            controls?.dispose();
            if (renderer) {
                renderer.domElement.removeEventListener('pointerdown', handlePointer);
                renderer.domElement.remove();
                renderer.dispose();
            }
            disposeObject(world);
        };
    }, [component, height, manifest]);

    return (
        <figure className="adk-ui-scene-3d" style={{ minHeight: height }}>
            {(component.title || component.description) && (
                <figcaption className="adk-ui-scene-3d__caption">
                    {component.title && <strong>{component.title}</strong>}
                    {component.description && <span>{component.description}</span>}
                </figcaption>
            )}
            {error ? (
                <div className="adk-ui-scene-3d__fallback" role="status">{error}</div>
            ) : (
                <div ref={mountRef} className="adk-ui-scene-3d__viewport" style={{ height }} />
            )}
            {labelledObjects.length > 0 && (
                <div className="adk-ui-scene-3d__actions" aria-label="3D scene objects">
                    {labelledObjects.map((object) => object.action_id ? (
                        <button key={object.id} type="button" onClick={() => onAction?.({ action: 'button_click', action_id: object.action_id!, source_component_id: object.id })}>
                            {object.label}
                        </button>
                    ) : <span key={object.id}>{object.label}</span>)}
                </div>
            )}
        </figure>
    );
}
