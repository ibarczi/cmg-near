import React from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './App'
import { Buffer } from 'buffer'
import { NearProvider } from './NearContext'

global.Buffer = Buffer // ??????????????

const container = document.getElementById('root')
const root = createRoot(container)
root.render(<NearProvider><App /></NearProvider>)


  


