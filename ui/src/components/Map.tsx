'use client'

import type { Map as MapType } from 'maplibre-gl'
import maplibregl from 'maplibre-gl'
import { PMTiles, Protocol } from 'pmtiles'
import { useEffect, useRef, useState } from 'react'

type PMTilesFile = {
  id: string
  name: string
  filename: string
  description: string
  type: 'points' | 'polygons'
}

const PMTILES_FILES: PMTilesFile[] = [
  { id: 'base', name: 'Base Points', filename: 'out.pmtiles', description: 'Original point data', type: 'points' },
  { id: 'buffer100', name: 'Buffer 100m', filename: 'out_buffer_100.pmtiles', description: '100m buffer polygons', type: 'polygons' },
  { id: 'buffer1000', name: 'Buffer 1000m', filename: 'out_buffer_1000.pmtiles', description: '1000m buffer polygons', type: 'polygons' }
]

export const Map = () => {
  const mapContainer = useRef<HTMLDivElement>(null)
  const map = useRef<MapType>(null)
  const [lng, setLng] = useState(8.4)
  const [lat, setLat] = useState(49)
  const [z, setZoom] = useState(12.25)
  const [selectedFile, setSelectedFile] = useState<PMTilesFile>(PMTILES_FILES[0])
  const [currentSourceId, setCurrentSourceId] = useState<string | null>(null)

  // Initialize PMTiles protocol
  useEffect(() => {
    const protocol = new Protocol()
    maplibregl.addProtocol('pmtiles', protocol.tile)
    return () => {
      maplibregl.removeProtocol('pmtiles')
    }
  }, [])

  useEffect(() => {
    if (map.current) return // initialize map only once
    if (!mapContainer.current) return
    map.current = new maplibregl.Map({
      container: mapContainer.current,
      style: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
      center: [lng, lat],
      zoom: z
    })
  }, [lat, lng, z])

  useEffect(() => {
    if (!map.current) return // wait for map to initialize
    map.current.on('move', () => {
      if (!map.current) return
      setLng(map.current.getCenter().lng)
      setLat(map.current.getCenter().lat)
      setZoom(map.current.getZoom())
    })
  }, [])

  // Clear current layer if exists
  const clearCurrentLayer = () => {
    if (!map.current || !currentSourceId) return

    const currentMap = map.current
    try {
      // Remove all possible layers for the current source
      const layers = [
        `${currentSourceId}-points`, 
        `${currentSourceId}-polygons`,
        `${currentSourceId}-polygons-stroke`
      ]
      layers.forEach(layerId => {
        if (currentMap.getLayer(layerId)) {
          currentMap.removeLayer(layerId)
        }
      })
      
      if (currentMap.getSource(currentSourceId)) {
        currentMap.removeSource(currentSourceId)
      }
    } catch (error) {
      console.error('Error clearing current layer:', error)
    }
  }

  // Load PMTiles file based on selected file
  const loadPMTilesFile = async (file: PMTilesFile) => {
    if (!map.current) return

    // Clear previous layer
    clearCurrentLayer()

    try {
      const pmtilesUrl = `pmtiles://${file.filename}`
      const sourceId = `pmtiles-${file.id}`
      const layerId = `${sourceId}-${file.type}`

      // Try to read PMTiles metadata to get layer information
      const pmtiles = new PMTiles(file.filename)
      const metadata = (await pmtiles.getMetadata()) as any

      console.log('metadata', metadata)

      // Get the first layer name (assuming single layer as per requirement)
      const sourceLayer =
        metadata?.vector_layers && metadata.vector_layers.length > 0 ? metadata.vector_layers[0].id : 'default'

      // Add the PMTiles source
      map.current.addSource(sourceId, {
        type: 'vector',
        url: pmtilesUrl
      })

      // Add appropriate layer based on file type
      if (file.type === 'points') {
        map.current.addLayer({
          id: layerId,
          type: 'circle',
          source: sourceId,
          'source-layer': sourceLayer,
          paint: {
            'circle-pitch-scale': 'map',
            'circle-radius': 4,
            'circle-opacity': 0.2,
            'circle-color': '#3b82f6'
          }
        })
      } else {
        // For polygons (buffer zones)
        map.current.addLayer({
          id: layerId,
          type: 'fill',
          source: sourceId,
          'source-layer': sourceLayer,
          paint: {
            'fill-color': file.id === 'buffer100' ? '#10b981' : '#f59e0b',
            'fill-opacity': 0.3
          }
        })
        
        // Add stroke for polygons
        map.current.addLayer({
          id: `${layerId}-stroke`,
          type: 'line',
          source: sourceId,
          'source-layer': sourceLayer,
          paint: {
            'line-color': file.id === 'buffer100' ? '#059669' : '#d97706',
            'line-width': 1,
            'line-opacity': 0.8
          }
        })
      }

      setCurrentSourceId(sourceId)
      console.log(`Loaded PMTiles file: ${file.filename} with layer: ${sourceLayer}`)

      // Zoom to the bounds of the data if available
      if (metadata?.bounds && Array.isArray(metadata.bounds) && metadata.bounds.length === 4) {
        const [minLng, minLat, maxLng, maxLat] = metadata.bounds
        map.current.fitBounds(
          [
            [minLng, minLat],
            [maxLng, maxLat]
          ],
          { padding: 50 }
        )
      }
    } catch (error) {
      console.error('Error loading PMTiles file:', error)
    }
  }

  // Switch to a different file
  const switchToFile = (file: PMTilesFile) => {
    setSelectedFile(file)
    if (map.current && map.current.isStyleLoaded()) {
      loadPMTilesFile(file)
    }
  }

  // Load initial PMTiles when map is ready
  useEffect(() => {
    if (!map.current) return
    
    map.current.on('load', () => {
      loadPMTilesFile(selectedFile)
    })
  }, [])

  // Load new file when selection changes
  useEffect(() => {
    if (map.current && map.current.isStyleLoaded()) {
      loadPMTilesFile(selectedFile)
    }
  }, [selectedFile])

  return (
    <div className="flex flex-col flex-grow">
      {/* Map container */}
      <div className="flex-grow relative" ref={mapContainer}></div>
      
      {/* Bottom switcher bar */}
      <div className="bg-gray-900 bg-opacity-90 backdrop-blur-sm border-t border-gray-700 p-4">
        <div className="flex items-center justify-center space-x-4">
          <span className="text-white text-sm font-medium">Choose renderer:</span>
          <div className="flex space-x-2">
            {PMTILES_FILES.map((file) => (
              <button
                key={file.id}
                onClick={() => switchToFile(file)}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 ${
                  selectedFile.id === file.id
                    ? 'bg-blue-600 text-white shadow-lg'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600 hover:text-white'
                }`}
              >
                <div className="flex items-center space-x-2">
                  <div
                    className={`w-3 h-3 rounded ${
                      file.type === 'points'
                        ? 'bg-blue-400'
                        : file.id === 'buffer100'
                        ? 'bg-emerald-400'
                        : 'bg-amber-400'
                    }`}
                  ></div>
                  <span>{file.name}</span>
                </div>
              </button>
            ))}
          </div>
          <div className="text-gray-400 text-xs w-xs">
            {selectedFile.description}
          </div>
        </div>
      </div>
    </div>
  )
}
