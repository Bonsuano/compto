# Here is how the struct works

TODO: Good name for this file

TODO: Add drawio for struct layout

TODO: Move other documentation to this file

TODO: Move all the user pda code to its own folder, maybe separate docs from code? Maybe not.





Possible States:  
- No Valid Hashes
- One Valid Hash
- Two Valid Hashes



State Transitions:   
(1) One valid hash -> Two valid hashes  
(2) One valid hash -> One valid hash  
(3) Two valid hashes -> One valid hash  
(4) Two valid hashes -> No valid hashes  
(5) No valid hashes -> One valid hash  

| Possible Events                                             | State Transitions |
|-------------------------------------------------------------|-------------------|
| (1) No valid hashes -> One valid hash                       | (5)               |
| (2) One valid hash -> Same valid hash                       | -                 |
| (3) One valid hash -> New valid hash                        | (2)               |
| (4) One valid hash -> Two valid hashes                      | (1)               |
| (5) Two valid hashes -> Same two valid hashes               | -                 |
| (6) Two valid hashes -> Only the second hash is valid       | (3)               |
| (7) Two valid hashes -> New valid hash                      | (4), (5)          |
| (8) Two valid hashes -> Second valid hash + new valid hash  | (3), (1)          |



![image](proof_flow.drawio.png)