use std::env;
use std::{error::Error};

use std::collections::HashMap;

use csv;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize)]
struct Client{
    type1: String,
    client:u16,
    tx:u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    amount:Option<f32>
}

#[derive(Debug,Clone,Serialize,Deserialize)]
struct Accounts{
    client:u16,
    available:f32,
    held:f32,
    total:f32,
    locked:bool
}

fn get_amount(x: Option<f32>) -> f32 {
    match x {
        None => 0.0,
        Some(i) => i,
    }
}

fn read_from_file(path: &str) -> Result<(),Box<dyn Error>>{
    let mut reader = csv::Reader::from_path(path)?;
    let mut map: HashMap<u16,Accounts>  = HashMap::new();
    let mut tran_map: HashMap<u32,f32> = HashMap::new();
    let mut dispute_map : HashMap<u16,bool> = HashMap::new();
    for result in reader.deserialize(){
        let record: Client =result?;
        let mut rec_amount = get_amount(record.amount);
        let  mut data:     Accounts =    Accounts { 
            client: record.client, 
            available: 0.0, 
            held: 0.0, 
            total: 0.0, 
            locked: false 
        };
        if map.contains_key(&record.client){
                data =  map.get(&record.client).unwrap().clone();
            }
        if record.type1 == "deposit"{
            data.available += rec_amount;
            data.total += rec_amount;
        }
        else if record.type1=="withdrawal" && data.available >= rec_amount{
            data.available -= rec_amount;
            data.total -= rec_amount;
        }else if tran_map.contains_key(&record.tx){
             let amt: f32 = tran_map.get(&record.tx).unwrap().clone();
             rec_amount = amt;
            if record.type1=="dispute" {
                data.available -= amt;
                data.held += amt;
                dispute_map.insert(record.client, true);
            }
            else if record.type1=="resolve" && dispute_map.contains_key(&record.client) {
                data.available += amt;
                data.held -= amt;
                dispute_map.remove(&record.client);            }
            else if record.type1=="chargeback" && dispute_map.contains_key(&record.client){
                data.total -= amt;
                data.held -= amt;
                data.locked = true;
            }
        }
        
        map.insert(data.client, data);
        tran_map.insert(record.tx, rec_amount);
    }
    
    println!("client,available,held,total,locked");
    for (key, value) in map {
        println!("{},{},{},{},{}",value.client,value.available,value.held,value.total,value.locked);
    }


    Ok(())
}


fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len()> 1{
        let path:&str = &("./".to_owned()+ &args[1]);
        if let Err(e) = read_from_file(path){
            println!("{}",e);
        }
    }else {
        if let Err(e) = read_from_file("./transactions.csv"){
            println!("{}",e);
        }
    }
    
}


