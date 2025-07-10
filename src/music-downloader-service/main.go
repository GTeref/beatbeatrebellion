package main

import (
	"log"
	"music-downloader/handlers"
	"music-downloader/utils"

	"github.com/gin-gonic/gin"
)

func main() {
	// Create Gin router
	r := gin.Default()

	// Enable CORS
	r.Use(utils.CORSMiddleware())

	//placeholder endpoints
	api := r.Group("/api/v1")
	{
		api.GET("/search", handlers.SearchMusic)
		api.POST("/download", handlers.DownloadMusic)
		api.GET("/download/status/:id", handlers.GetDownloadStatus)
		api.GET("/download/file/:id", handlers.GetDownloadedFile)
	}

	log.Println("Music Downloader Service starting on :8080")
	if err := r.Run(":8080"); err != nil {
		log.Fatal("Failed to start server:", err)
	}
}
