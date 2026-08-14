import type { ApplicationShowcaseExample } from '../types';
import { spatialKit } from '../kits';
import { spatialLogisticsMeta } from '../manifest';

export const spatialLogisticsExample: ApplicationShowcaseExample = {
  ...spatialLogisticsMeta,
  kit: spatialKit,
  kind: 'application',
  application: {
    id: 'orbit-spatial-logistics',
    name: 'Orbit',
    kit_id: spatialKit.id,
    initial_route: '/',
    navigation: [{ label: 'Digital twin', route: '/' }, { label: 'Exceptions', route: '/exceptions' }],
    pages: [
      {
        id: 'twin',
        route: '/',
        title: 'Fulfilment floor twin',
        template: 'immersive',
        atmosphere: 'spatial',
        eyebrow: 'FC-07 / live topology / 14:08 UTC',
        hero: {
          title: 'Reason about the floor as a system.',
          description: 'Select a labelled zone to send a structured action back to the agent. The host owns camera controls, rendering limits, materials, and asset policy.',
          alignment: 'split',
          actions: [{ type: 'button', label: 'Open ranked exceptions', action_id: 'navigate:/exceptions', variant: 'outline' }],
          visual: [{
            type: 'scene_3d',
            title: 'Live warehouse topology',
            description: 'Orbit to inspect. Select a zone below to focus the agent.',
            height: 560,
            background: 'contrast',
            auto_rotate: true,
            controls: true,
            camera: { position: [6.5, 5.2, 8.5], target: [0, 0, 0], fov: 38 },
            objects: [
              { kind: 'primitive', id: 'sortation', shape: 'torus', position: [0, 0.2, 0], rotation: [1.57, 0, 0], scale: [1.8, 1.8, 1.8], material: 'primary', label: 'Sortation loop', action_id: 'focus_sortation' },
              { kind: 'primitive', id: 'inbound', shape: 'box', position: [-2.9, -0.45, 0.4], scale: [1.5, 0.55, 2.4], material: 'info', label: 'Inbound staging', action_id: 'focus_inbound' },
              { kind: 'primitive', id: 'packing', shape: 'box', position: [2.9, -0.45, 0.5], scale: [1.6, 0.55, 2.2], material: 'warning', label: 'Packing zone', action_id: 'focus_packing' },
              { kind: 'primitive', id: 'robot-aisle-a', shape: 'cylinder', position: [-1.5, 0.2, -2.5], scale: [0.45, 1.4, 0.45], material: 'success', label: 'Robot aisle A', action_id: 'focus_robot_a' },
              { kind: 'primitive', id: 'robot-aisle-b', shape: 'cylinder', position: [0, 0.2, -2.5], scale: [0.45, 1.4, 0.45], material: 'success', label: 'Robot aisle B', action_id: 'focus_robot_b' },
              { kind: 'primitive', id: 'robot-aisle-c', shape: 'cylinder', position: [1.5, 0.2, -2.5], scale: [0.45, 1.4, 0.45], material: 'muted', label: 'Robot aisle C', action_id: 'focus_robot_c' },
              { kind: 'primitive', id: 'floor', shape: 'plane', position: [0, -1.05, 0], rotation: [-1.57, 0, 0], scale: [5.2, 5.2, 5.2], material: 'muted' },
            ],
            fallback: 'The 3D twin is unavailable on this device. Open Exceptions for the same ranked operational actions.',
          }],
        },
        regions: [{
          id: 'spatial-signals',
          title: 'Scene-linked operating signals',
          description: '3D remains part of a usable workflow, not a decorative detour.',
          layout: 'stats',
          columns: 3,
          components: [
            { type: 'card', title: 'Packing', description: 'Queue above target', content: [{ type: 'progress', value: 86, label: 'Utilization' }, { type: 'button', label: 'Focus packing', action_id: 'focus_packing', variant: 'outline' }] },
            { type: 'card', title: 'Sortation', description: 'Nominal throughput', content: [{ type: 'progress', value: 64, label: 'Utilization' }, { type: 'button', label: 'Focus sortation', action_id: 'focus_sortation', variant: 'outline' }] },
            { type: 'card', title: 'Robot aisle C', description: '4 units awaiting recovery', content: [{ type: 'badge', label: 'Action needed', variant: 'warning' }, { type: 'button', label: 'Inspect aisle C', action_id: 'focus_robot_c', variant: 'outline' }] },
          ],
        }],
      },
      {
        id: 'exceptions',
        route: '/exceptions',
        title: 'Ranked floor exceptions',
        template: 'dashboard',
        atmosphere: 'grid',
        eyebrow: 'Agent-ranked by throughput impact',
        regions: [{
          id: 'exception-list',
          title: 'Intervene where recovery compounds',
          layout: 'split',
          columns: 2,
          components: [
            {
              type: 'table',
              sortable: true,
              columns: [{ header: 'Zone', accessor_key: 'zone' }, { header: 'Exception', accessor_key: 'issue' }, { header: 'Impact', accessor_key: 'impact' }, { header: 'Owner', accessor_key: 'owner' }],
              data: [
                { zone: 'Packing', issue: 'Queue +22%', impact: '1,840 orders', owner: 'Shift lead' },
                { zone: 'Aisle C', issue: '4 robots paused', impact: '620 picks/hr', owner: 'Automation' },
                { zone: 'Inbound', issue: 'Dock 7 delayed', impact: '38 pallets', owner: 'Receiving' },
              ],
            },
            { type: 'key_value', pairs: [{ key: 'Scene contract', value: 'Bounded primitives + approved GLB' }, { key: 'Object budget', value: '64 objects / 8 models' }, { key: 'Renderer', value: 'WebGPU, WebGL 2 fallback' }, { key: 'Accessibility', value: 'Text fallback + labelled actions' }] },
          ],
        }],
        footer: [{ type: 'button', label: 'Return to digital twin', action_id: 'navigate:/', variant: 'outline' }],
      },
    ],
  },
};
