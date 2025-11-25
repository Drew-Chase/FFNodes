import { TMDBMovie, TMDBTVShow, TMDBResponse, TMDBPosterItem } from '../types/tmdb';

const TMDB_API_KEY = '378ae44c6e7f5dde094cd8c8456378e0';
const TMDB_BASE_URL = 'https://api.themoviedb.org/3';
const TMDB_IMAGE_BASE_URL = 'https://image.tmdb.org/t/p';

export class TMDBService {
  private static async fetchFromTMDB<T>(endpoint: string): Promise<T> {
    const url = `${TMDB_BASE_URL}${endpoint}${endpoint.includes('?') ? '&' : '?'}api_key=${TMDB_API_KEY}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`TMDB API error: ${response.statusText}`);
    }

    return response.json();
  }

  static getImageUrl(path: string | null, size: 'w92' | 'w154' | 'w185' | 'w342' | 'w500' | 'w780' | 'original' = 'w500'): string | null {
    if (!path) return null;
    return `${TMDB_IMAGE_BASE_URL}/${size}${path}`;
  }

  static async getPopularMovies(page: number = 1): Promise<TMDBResponse<TMDBMovie>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBMovie>>(`/movie/popular?page=${page}`);
  }

  static async getTrendingMovies(timeWindow: 'day' | 'week' = 'week'): Promise<TMDBResponse<TMDBMovie>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBMovie>>(`/trending/movie/${timeWindow}`);
  }

  static async getTopRatedMovies(page: number = 1): Promise<TMDBResponse<TMDBMovie>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBMovie>>(`/movie/top_rated?page=${page}`);
  }

  static async getNowPlayingMovies(page: number = 1): Promise<TMDBResponse<TMDBMovie>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBMovie>>(`/movie/now_playing?page=${page}`);
  }

  static async getPopularTVShows(page: number = 1): Promise<TMDBResponse<TMDBTVShow>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBTVShow>>(`/tv/popular?page=${page}`);
  }

  static async getTrendingTVShows(timeWindow: 'day' | 'week' = 'week'): Promise<TMDBResponse<TMDBTVShow>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBTVShow>>(`/trending/tv/${timeWindow}`);
  }

  static async getTopRatedTVShows(page: number = 1): Promise<TMDBResponse<TMDBTVShow>> {
    return this.fetchFromTMDB<TMDBResponse<TMDBTVShow>>(`/tv/top_rated?page=${page}`);
  }

  static async getMixedPopularPosters(moviePages: number = 3, tvPages: number = 2): Promise<TMDBPosterItem[]> {
    const posters: TMDBPosterItem[] = [];

    try {
      // Fetch multiple pages of movies
      const moviePromises = Array.from({ length: moviePages }, (_, i) =>
        this.getPopularMovies(i + 1)
      );

      // Fetch multiple pages of TV shows
      const tvPromises = Array.from({ length: tvPages }, (_, i) =>
        this.getPopularTVShows(i + 1)
      );

      const [movieResponses, tvResponses] = await Promise.all([
        Promise.all(moviePromises),
        Promise.all(tvPromises),
      ]);

      // Process movies
      for (const response of movieResponses) {
        for (const movie of response.results) {
          if (movie.poster_path) {
            posters.push({
              id: movie.id,
              title: movie.title,
              posterUrl: this.getImageUrl(movie.poster_path, 'w500') || '',
              type: 'movie',
            });
          }
        }
      }

      // Process TV shows
      for (const response of tvResponses) {
        for (const show of response.results) {
          if (show.poster_path) {
            posters.push({
              id: show.id,
              title: show.name,
              posterUrl: this.getImageUrl(show.poster_path, 'w500') || '',
              type: 'tv',
            });
          }
        }
      }

      // Shuffle the posters for variety
      return this.shuffleArray(posters);
    } catch (error) {
      console.error('Error fetching TMDB posters:', error);
      return [];
    }
  }

  static async getTrendingPosters(): Promise<TMDBPosterItem[]> {
    const posters: TMDBPosterItem[] = [];

    try {
      const [movieResponse, tvResponse] = await Promise.all([
        this.getTrendingMovies('week'),
        this.getTrendingTVShows('week'),
      ]);

      // Process movies
      for (const movie of movieResponse.results) {
        if (movie.poster_path) {
          posters.push({
            id: movie.id,
            title: movie.title,
            posterUrl: this.getImageUrl(movie.poster_path, 'w500') || '',
            type: 'movie',
          });
        }
      }

      // Process TV shows
      for (const show of tvResponse.results) {
        if (show.poster_path) {
          posters.push({
            id: show.id,
            title: show.name,
            posterUrl: this.getImageUrl(show.poster_path, 'w500') || '',
            type: 'tv',
          });
        }
      }

      return this.shuffleArray(posters);
    } catch (error) {
      console.error('Error fetching trending posters:', error);
      return [];
    }
  }

  private static shuffleArray<T>(array: T[]): T[] {
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
  }
}
