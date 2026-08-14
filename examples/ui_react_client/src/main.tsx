import { lazy, StrictMode, Suspense } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import { resolveShowcaseExampleId } from './showcase/route'

const App = lazy(() => import('./App.tsx'))
const ShowcasePage = lazy(() => import('./showcase/ShowcasePage').then((module) => ({ default: module.ShowcasePage })))

const showcaseExampleId = resolveShowcaseExampleId()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Suspense fallback={<div className="route-loading">Loading interface...</div>}>
      {showcaseExampleId ? <ShowcasePage exampleId={showcaseExampleId} /> : <App />}
    </Suspense>
  </StrictMode>,
)
