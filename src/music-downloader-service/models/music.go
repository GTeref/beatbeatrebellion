package models

type SearchResult struct {
	ID           string `json:"id"`
	Title        string `json:"title"`
	Artist       string `json:"artist"`
	Duration     string `json:"duration"`
	ThumbnailURL string `json:"thumbnail_url"`
	Source       string `json:"source"` // "youtube", "spotify"
	URL          string `json:"url"`
	Quality      string `json:"quality"`
}

type DownloadRequest struct {
	URL     string `json:"url" binding:"required"`
	Quality string `json:"quality,omitempty"` // "highest", "medium", "low"
	Format  string `json:"format,omitempty"`  // "mp3", "wav", "m4a"
}

type DownloadResponse struct {
	ID       string `json:"id"`
	Status   string `json:"status"` // "queued", "downloading", "completed", "failed"
	Progress int    `json:"progress"`
	Message  string `json:"message,omitempty"`
	FileURL  string `json:"file_url,omitempty"`
}

type DownloadStatus struct {
	ID       string `json:"id"`
	Status   string `json:"status"`
	Progress int    `json:"progress"`
	Message  string `json:"message"`
	FileURL  string `json:"file_url,omitempty"`
	FileSize int64  `json:"file_size,omitempty"`
	Filename string `json:"filename,omitempty"`
}
