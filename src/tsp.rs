extern crate rand;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use self::rand::{Rng, thread_rng};
use self::rand::seq::SliceRandom;
use self::rand::distributions::{Distribution, Uniform};

pub struct City {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct Path {
    fitness: f64,
    order: Vec<usize>
}

impl Path {
    pub fn calculate_fitness(path: &Vec<usize>, city_list: &Vec<City>) -> f64 {
        let path_length = city_list.len();
        let mut cost = 0.0;
        for i in 0..path_length - 2 {
            let a = &city_list[path[i]];
            let b = &city_list[path[i + 1]];
            cost = cost + ((a.x - b.x).powf(2.0) + (a.y - b.y).powf(2.0)).sqrt();
        }
        1.0 / cost
    }
    
    pub fn breed(&self, other: &Path, city_list: &Vec<City>) -> Path {
        let mut rng = thread_rng();
        let crossover_type = Uniform::new(0, 3).sample(&mut rng);
        let order = match crossover_type {
            0 => Path::single_point_crossover(&self.order, &other.order),
            1 => Path::uniform_order_crossover(&self.order, &other.order),
            _ => Path::partially_mapped_crossover(&self.order, &other.order),
        };
        let fitness = Path::calculate_fitness(&order, city_list);
        Path { fitness, order }
    }
    
    fn single_point_crossover(mother: &Vec<usize>, father: &Vec<usize>) -> Vec<usize> {
        let mut rng = thread_rng();
        let crossover_point = Uniform::new(0, mother.len()).sample(&mut rng);
        let mother_dna = &mother[0..crossover_point];
        let father_dna: Vec<usize> = father.iter().filter_map(|d| {
            if !mother_dna.contains(d) {
                return Some(*d)
            }
            None
        }).collect();
        let mut child = mother_dna.to_vec();
        child.extend(father_dna);
        child
    }

    fn uniform_order_crossover(mother: &Vec<usize>, father: &Vec<usize>) -> Vec<usize> {
        let mut rng = thread_rng();
        let mut child: Vec<usize> = vec![0; mother.len()];
        let mut positions: Vec<usize> = (0..mother.len()).collect();
        positions.shuffle(&mut rng);

        // Copy a subset of gene positions from mother to child
        for &position in positions.iter().take(mother.len() / 2) {
            child[position] = mother[position];
        }

        // Fill the remaining positions with genes from father in the order they appear
        let mut father_index = 0;
        for position in 0..father.len() {
            if child[position] == 0 {
                while father_index < father.len() && child.contains(&father[father_index]) {
                    father_index += 1;
                }
                if father_index < father.len() {
                    child[position] = father[father_index];
                }
            }
        }
        child
    }

    fn partially_mapped_crossover(mother: &Vec<usize>, father: &Vec<usize>) -> Vec<usize> {
        let mut rng = thread_rng();
        let crossover_point1 = Uniform::new(0, mother.len()).sample(&mut rng);
        let crossover_point2 = Uniform::new(0, mother.len()).sample(&mut rng);
        let (start, end) = if crossover_point1 < crossover_point2 {
            (crossover_point1, crossover_point2)
        } else {
            (crossover_point2, crossover_point1)
        };
    
        let mut child: Vec<usize> = vec![0; mother.len()];
        let mut gene_mapping: HashMap<usize, usize> = HashMap::new();
        let mut gene_set: HashSet<usize> = HashSet::new();
    
        for i in start..end {
            child[i] = mother[i];
            gene_mapping.insert(mother[i], father[i]);
            gene_set.insert(mother[i]);
        }
    
        for i in (0..start).chain(end..mother.len()) {
            let mut gene = father[i];
            while gene_set.contains(&gene) {
                gene = *gene_mapping.get(&gene).unwrap_or(&gene);
            }
            child[i] = gene;
            gene_set.insert(gene);
        }
        child
    }

    pub fn mutate(&mut self, city_list: &Vec<City>) {
      let mut rng = thread_rng();
      let point_one = Uniform::new(0, self.order.len()).sample(&mut rng);
      let point_two = Uniform::new(0, self.order.len()).sample(&mut rng);
      self.order.swap(point_one, point_two);
      self.fitness = Path::calculate_fitness(&self.order, &city_list);
    }
}

pub struct Simulation {
     city_list: Vec<City>,
     population: Vec<Path>,
     best_paths: Vec<Path>,
     iterations: usize,
     crossover_rate: f64,
     mutation_rate: f64,
     survival_rate: f64,
 }

impl Simulation {
    pub fn new(city_list: Vec<City>,
               population_size: usize,
               iterations: usize,
               crossover_rate: f64,
               mutation_rate: f64,
               survival_rate: f64,) -> Self {

        let population = Self::initial_population(&city_list, population_size);
        let best_paths = Vec::with_capacity(population_size);

        Simulation {
            city_list,
            population,
            best_paths,
            iterations,
            crossover_rate,
            mutation_rate,
            survival_rate,
        }
    }

    fn initial_population(city_list: &Vec<City>, population_size: usize) -> Vec<Path> {
        let base_list: Vec<usize> = (0..city_list.len()).collect();
        let mut rng = thread_rng();
        let mut population: Vec<Path> = Vec::new();
    
        for _ in 0..population_size {
            let mut p = base_list.clone();
            p.shuffle(&mut rng);
            let fitness = Path::calculate_fitness(&p, city_list);
    
            population.push(Path { fitness, order: p });
        }
        population
    }
    
    pub fn run(&mut self) -> () {
        let mut fittest = self.find_fittest();
        println!("Starting {} iterations...", self.iterations);
        for _ in 0..self.iterations {
            self.generate_children();
            let challenger = self.find_fittest();
            if challenger.fitness > fittest.fitness {
                fittest = challenger;
            }
            self.best_paths.push(fittest.clone());
        }
        println!("Champion:\n{:?}", fittest);
    }

    fn find_fittest(&self) -> Path {
        let mut fittest = &self.population[0];

        for i in 1..self.population.len() {
            let p = &self.population[i];
            if p.fitness > fittest.fitness {
                fittest = p;
            }
        }
        return fittest.clone();
    }

    fn generate_children(&mut self) {
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        let breeding_count = (self.population.len() as f64 * self.crossover_rate) as usize;
        let surviving_parent_count = (breeding_count as f64 * self.survival_rate) as usize;

        let mut breeding_population = Vec::new();
        breeding_population.extend_from_slice(&self.population[0..breeding_count]);

        let mut offspring = Vec::new();

        let mut rng = thread_rng();
        let pcnt_range = Uniform::new(0, breeding_population.len());
        for i in 0..self.population.len() - surviving_parent_count - 2 {
            let rs = pcnt_range.sample(&mut rng);
            offspring.push(
                breeding_population[i % breeding_population.len()].breed(
                    &breeding_population[rs],
                    &self.city_list
                )
            );
        }

        let mut next_generation = Vec::new();
        next_generation.extend_from_slice(&self.population[0..surviving_parent_count]);
        next_generation.append(&mut offspring);
        // Add a few weak individuals to keep the genetic diversity higher
        next_generation.extend_from_slice(&self.population[self.population.len() - 2..self.population.len()]);

        assert!(next_generation.len() == self.population.len());

        for p in 0..next_generation.len() {
            if thread_rng().gen_bool(self.mutation_rate) {
                next_generation[p].mutate(&self.city_list);
            }
        }
        self.population = next_generation;
    }

    fn write_best_path_csv(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(file_path)?;
        for (iteration, path) in self.best_paths.iter().enumerate() {
            let path_str = path
                .order
                .iter()
                .map(|&index| index.to_string())
                .collect::<Vec<String>>()
                .join(",");
            writeln!(file, "{},{}", iteration, path_str)?;
        }
        Ok(())
    }
}
