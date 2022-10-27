fn find_split_index(data:&[u8],pattern:&[u8]) -> Option<usize> {

    if pattern == b"INPUT" {
        
        let pos = 
        data.windows(pattern.len())
            .enumerate()
            .find(|(_, w)| matches!(*w, b"INPUT"))
            .map(|(i, _)| i);
        return pos
    }
    
    if pattern == b"OUTPUT" {
        let pos = 
        data.windows(pattern.len())
            .enumerate()
            .find(|(_, w)| matches!(*w, b"OUTPUT"))
            .map(|(i, _)| i);
            
        return pos
    }
    return None  
    }